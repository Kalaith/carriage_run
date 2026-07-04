//! Mission input, route progress, and spawn flow.

use super::*;
use crate::data::MissionDef;
use macroquad::prelude::*;

impl MissionRun {
    pub fn handle_input(&mut self, input: MissionInput) {
        if input.repair_pressed {
            self.use_emergency_repair();
        }

        if input.pressed && contains_point(input.play_rect, input.mouse) {
            if self.carriage.rect().contains(input.mouse) {
                self.drag = DragState::Carriage;
            } else if let Some(guard) = self
                .guards
                .iter()
                .find(|guard| guard.is_active() && guard.pos.distance(input.mouse) <= 28.0)
            {
                self.drag = DragState::Guard { guard_id: guard.id };
            }
        }

        match self.drag {
            DragState::Carriage if input.down => {
                self.carriage.target_x = self.clamp_carriage_x(input.mouse.x);
            }
            DragState::Guard { guard_id } if input.released => {
                self.issue_guard_order(guard_id, input.mouse);
                self.drag = DragState::None;
            }
            _ => {}
        }

        if input.released && matches!(self.drag, DragState::Carriage) {
            self.drag = DragState::None;
        }
    }

    pub fn update(&mut self, mission: &MissionDef, dt: f32) -> Option<MissionReport> {
        self.elapsed += dt;
        self.alert.update(dt);
        self.carriage.update_timers(dt);
        self.handle_keyboard(dt);
        self.update_carriage(dt);
        self.update_spawns(dt);
        self.update_hazards(dt);
        self.update_guard_orders(dt);
        self.update_enemies(dt);
        self.update_shots(dt);
        self.handle_hazard_collisions(dt);
        self.update_mission_pressure(dt);
        self.cleanup_entities();

        if self.carriage.health <= 0.0 {
            Some(self.make_report(mission, false, "Carriage destroyed"))
        } else if self.carriage.cargo <= 0.0 {
            Some(self.make_report(mission, false, "Cargo lost"))
        } else if self.mission_kind == MissionKind::PrisonerEscort && self.special_meter >= 100.0 {
            Some(self.make_report(mission, false, "Prisoner escaped"))
        } else if self.mission_kind == MissionKind::PrincessEscort && self.special_meter <= 0.0 {
            Some(self.make_report(mission, false, "Passenger safety failed"))
        } else if self.mission_kind == MissionKind::MedicineRun && self.special_meter <= 0.0 {
            Some(self.make_report(mission, false, "Medicine spoiled"))
        } else if self.mission_kind == MissionKind::MonsterEggTransport && self.special_meter <= 0.0
        {
            Some(self.make_report(mission, false, "Egg destabilized"))
        } else if self.mission_kind == MissionKind::RefugeeEscort && self.special_meter <= 0.0 {
            Some(self.make_report(mission, false, "Refugees scattered"))
        } else if self.mission_kind == MissionKind::RoyalBanquetSupplies
            && self.special_meter <= 0.0
        {
            Some(self.make_report(mission, false, "Banquet supplies ruined"))
        } else if self.mission_kind == MissionKind::SiegeSupplyRun && self.special_meter <= 0.0 {
            Some(self.make_report(mission, false, "Siege momentum lost"))
        } else if self.time_limit.is_some_and(|limit| self.elapsed >= limit) {
            Some(self.make_report(mission, false, "Delivery deadline missed"))
        } else if self.progress >= self.distance {
            Some(self.make_report(mission, true, "Destination reached"))
        } else {
            None
        }
    }

    fn progress_speed(&self) -> f32 {
        let base = match self.mission_kind {
            MissionKind::SiegeSupplyRun => 15.2,
            MissionKind::TimeDelivery => 18.3,
            _ => 17.5,
        };
        (base + self.wheel_bonus) * self.speed_factor()
    }

    fn issue_guard_order(&mut self, guard_id: u32, point: Vec2) {
        let carriage_near = point.distance(self.carriage.pos) < 70.0;
        let slot_target = self.ranged_slot_at(point);
        let enemy_target = self
            .enemies
            .iter()
            .filter(|enemy| enemy.is_active())
            .find(|enemy| enemy.pos.distance(point) <= enemy.radius + 22.0)
            .map(|enemy| enemy.id);

        let slot_available = slot_target.is_some_and(|slot| {
            self.guards
                .iter()
                .all(|guard| guard.id == guard_id || guard.mounted_slot != Some(slot))
        });

        if let Some(guard) = self.guards.iter_mut().find(|guard| guard.id == guard_id) {
            if guard.kind.is_ranged() && slot_available {
                guard.mounted_slot = slot_target;
                guard.order = GuardOrder::Escort;
                return;
            }

            if guard.kind.is_ranged() {
                guard.mounted_slot = None;
            }

            guard.order = if let Some(enemy_id) = enemy_target {
                GuardOrder::Attack(enemy_id)
            } else if carriage_near {
                GuardOrder::Escort
            } else {
                GuardOrder::Move(clamp_to_field(point))
            };
        }
    }

    fn ranged_slot_at(&self, point: Vec2) -> Option<usize> {
        (0..self.ranged_slots).find(|slot| point.distance(self.carriage_slot_pos(*slot)) < 34.0)
    }

    fn handle_keyboard(&mut self, dt: f32) {
        let mut axis = 0.0;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            axis -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            axis += 1.0;
        }
        if axis != 0.0 {
            self.carriage.target_x =
                self.clamp_carriage_x(self.carriage.target_x + axis * 270.0 * dt);
        }
    }

    fn update_carriage(&mut self, dt: f32) {
        let previous_center = road_center_at_y(CARRIAGE_Y, self.progress);
        let scroll_step = self.scroll_speed() * dt;
        self.progress += self.progress_speed() * dt;
        self.road_scroll = (self.road_scroll + scroll_step) % 96.0;
        self.terrain_scroll = (self.terrain_scroll + scroll_step) % 1_000_000.0;
        let center_delta = road_center_at_y(CARRIAGE_Y, self.progress) - previous_center;
        self.carriage.target_x = self.clamp_carriage_x(self.carriage.target_x + center_delta);
        self.carriage.pos.x = self.clamp_carriage_x(self.carriage.pos.x + center_delta);
        let response = 1.0 - (-5.0 * dt).exp();
        self.carriage.pos.x += (self.carriage.target_x - self.carriage.pos.x) * response;
        self.carriage.pos.x = self.clamp_carriage_x(self.carriage.pos.x);
    }

    fn update_spawns(&mut self, dt: f32) {
        if self.progress_ratio() >= 0.96 {
            return;
        }

        let grace = self.early_grace_multiplier();

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 && !self.enemy_mix.is_empty() {
            self.spawn_enemy();
            self.spawn_timer =
                (self.rng_range(1.4, 2.8) * grace / self.difficulty.max(0.75)).max(0.85);
        }

        self.hazard_timer -= dt;
        if self.hazard_timer <= 0.0 && !self.hazard_mix.is_empty() {
            self.spawn_hazard();
            self.hazard_timer =
                (self.rng_range(2.1, 4.2) * grace / self.difficulty.max(0.8)).max(1.4);
        }
    }

    /// Eases spawn pressure at the start of a route so the player can settle
    /// guards before threats ramp up. Returns 1.6x spacing at the start,
    /// tapering to 1.0x by the time 25% of the route is covered.
    fn early_grace_multiplier(&self) -> f32 {
        let ratio = self.progress_ratio();
        if ratio >= 0.25 {
            1.0
        } else {
            1.6 - 0.6 * (ratio / 0.25)
        }
    }

    fn spawn_enemy(&mut self) {
        let index = (self.rng.next_u64() as usize) % self.enemy_mix.len();
        let kind = EnemyKind::from_id(&self.enemy_mix[index]).unwrap_or(EnemyKind::Wolf);
        let side = (self.rng.next_u64() % 3) as u32;
        let pos = match side {
            0 => {
                let y = PLAY_TOP - 36.0;
                vec2(self.road_x_at(y, 30.0), y)
            }
            1 => {
                let y = self.rng_range(150.0, 450.0);
                vec2(
                    road_left_at_y(y, self.progress) - self.rng_range(42.0, 105.0),
                    y,
                )
            }
            _ => {
                let y = self.rng_range(150.0, 450.0);
                vec2(
                    road_right_at_y(y, self.progress) + self.rng_range(42.0, 105.0),
                    y,
                )
            }
        };

        self.enemies
            .push(Enemy::new(self.next_enemy_id, kind, pos, self.difficulty));
        self.next_enemy_id += 1;
    }

    fn spawn_hazard(&mut self) {
        let index = (self.rng.next_u64() as usize) % self.hazard_mix.len();
        let kind = HazardKind::from_id(&self.hazard_mix[index]).unwrap_or(HazardKind::Mud);
        let y = PLAY_TOP - 48.0;
        let pos = vec2(self.road_x_at(y, 70.0), y);
        self.hazards.push(Hazard::new(kind, pos));
    }

    fn update_hazards(&mut self, dt: f32) {
        let scroll = self.scroll_speed();
        for hazard in &mut self.hazards {
            let previous_center = road_center_at_y(hazard.pos.y, self.progress);
            hazard.pos.y += scroll * dt;
            let next_center = road_center_at_y(hazard.pos.y, self.progress);
            hazard.pos.x += next_center - previous_center;
        }
    }

    fn rng_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.rng.next_f32()
    }

    fn clamp_carriage_x(&self, x: f32) -> f32 {
        clamp_x_to_road_at_y(x, CARRIAGE_Y, self.progress, 45.0)
    }

    fn road_x_at(&mut self, y: f32, margin: f32) -> f32 {
        let left = road_left_at_y(y, self.progress) + margin;
        let right = road_right_at_y(y, self.progress) - margin;
        self.rng_range(left, right)
    }
}
