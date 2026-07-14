//! Guard, enemy, hazard, and damage updates for active missions.

use super::*;
use macroquad::prelude::*;

struct PendingGuardHit {
    kind: GuardKind,
    stars: u8,
    enemy_id: u32,
    enemy_kind: EnemyKind,
    damage: f32,
    origin: Vec2,
    target: Vec2,
}

impl MissionRun {
    pub(super) fn update_guard_orders(&mut self, dt: f32) {
        let enemies: Vec<(u32, Vec2, f32, EnemyKind)> = self
            .enemies
            .iter()
            .filter(|enemy| enemy.is_active())
            .map(|enemy| (enemy.id, enemy.pos, enemy.radius, enemy.kind))
            .collect();
        let escort_positions = self.escort_positions();
        let slot_positions: Vec<Vec2> = (0..self.ranged_slots)
            .map(|slot| self.carriage_slot_pos(slot))
            .collect();
        let carriage_pos = self.carriage.pos;
        let mut pending_hits = Vec::new();

        for (index, guard) in self.guards.iter_mut().enumerate() {
            guard.cooldown = (guard.cooldown - dt).max(0.0);
            guard.hit_flash = (guard.hit_flash - dt).max(0.0);
            guard.attack_flash = (guard.attack_flash - dt).max(0.0);

            if !guard.is_active() {
                continue;
            }

            if let Some(slot) = guard.mounted_slot {
                if let Some(pos) = slot_positions.get(slot) {
                    guard.pos = *pos;
                }
                if guard.cooldown <= 0.0 {
                    if let Some((enemy_id, target, _, kind)) =
                        nearest_enemy_in_range(&enemies, guard.pos, guard.range)
                    {
                        pending_hits.push(PendingGuardHit {
                            kind: guard.kind,
                            stars: guard.star_level,
                            enemy_id,
                            enemy_kind: kind,
                            damage: guard_hit_damage(guard, kind),
                            origin: guard.pos,
                            target,
                        });
                        guard.cooldown = guard.attack_cooldown;
                        guard.attack_flash = 0.16;
                    }
                }
                continue;
            }

            match guard.order.clone() {
                GuardOrder::Escort => {
                    guard.pos = move_towards(guard.pos, escort_positions[index], guard.speed * dt);
                    if guard.cooldown <= 0.0 {
                        if let Some((enemy_id, target, _, kind)) =
                            nearest_enemy_in_range(&enemies, guard.pos, auto_attack_range(guard))
                        {
                            pending_hits.push(PendingGuardHit {
                                kind: guard.kind,
                                stars: guard.star_level,
                                enemy_id,
                                enemy_kind: kind,
                                damage: guard_hit_damage(guard, kind),
                                origin: guard.pos,
                                target,
                            });
                            guard.cooldown = guard.attack_cooldown;
                            guard.attack_flash = 0.16;
                        }
                    }
                }
                GuardOrder::Roam => {
                    // Chase the nearest threat inside the carriage leash radius,
                    // otherwise ease back into escort formation.
                    let leash_target = enemies
                        .iter()
                        .filter(|(_, pos, _, _)| pos.distance(carriage_pos) <= ROAM_LEASH_RADIUS)
                        .min_by(|(_, a, _, _), (_, b, _, _)| {
                            a.distance(guard.pos)
                                .partial_cmp(&b.distance(guard.pos))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .copied();

                    if let Some((enemy_id, target, radius, kind)) = leash_target {
                        if guard.pos.distance(target) > guard.range + radius {
                            guard.pos = move_towards(guard.pos, target, guard.speed * dt);
                        } else if guard.cooldown <= 0.0 {
                            pending_hits.push(PendingGuardHit {
                                kind: guard.kind,
                                stars: guard.star_level,
                                enemy_id,
                                enemy_kind: kind,
                                damage: guard_hit_damage(guard, kind),
                                origin: guard.pos,
                                target,
                            });
                            guard.cooldown = guard.attack_cooldown;
                            guard.attack_flash = 0.16;
                        }
                    } else {
                        guard.pos =
                            move_towards(guard.pos, escort_positions[index], guard.speed * dt);
                    }
                }
                GuardOrder::Move(target) => {
                    guard.pos = move_towards(guard.pos, target, guard.speed * dt);
                    if guard.pos.distance(target) < 5.0 {
                        guard.order = GuardOrder::Hold;
                    }
                }
                GuardOrder::Hold => {
                    if guard.cooldown <= 0.0 {
                        if let Some((enemy_id, target, _, kind)) =
                            nearest_enemy_in_range(&enemies, guard.pos, auto_attack_range(guard))
                        {
                            pending_hits.push(PendingGuardHit {
                                kind: guard.kind,
                                stars: guard.star_level,
                                enemy_id,
                                enemy_kind: kind,
                                damage: guard_hit_damage(guard, kind),
                                origin: guard.pos,
                                target,
                            });
                            guard.cooldown = guard.attack_cooldown;
                            guard.attack_flash = 0.16;
                        }
                    }
                }
                GuardOrder::Attack(enemy_id) => {
                    let Some((_, target, radius, kind)) =
                        enemies.iter().find(|(id, _, _, _)| *id == enemy_id)
                    else {
                        guard.order = guard.home_stance();
                        continue;
                    };
                    if guard.pos.distance(*target) > guard.range + *radius {
                        guard.pos = move_towards(guard.pos, *target, guard.speed * dt);
                    } else if guard.cooldown <= 0.0 {
                        pending_hits.push(PendingGuardHit {
                            kind: guard.kind,
                            stars: guard.star_level,
                            enemy_id,
                            enemy_kind: *kind,
                            damage: guard_hit_damage(guard, *kind),
                            origin: guard.pos,
                            target: *target,
                        });
                        guard.cooldown = guard.attack_cooldown;
                        guard.attack_flash = 0.16;
                    }
                }
            }
        }

        for hit in pending_hits {
            self.apply_guard_hit(hit);
        }
    }

    pub(super) fn update_enemies(&mut self, dt: f32) {
        let guards: Vec<(u32, Vec2, f32)> = self
            .guards
            .iter()
            .filter(|guard| guard.is_active())
            .map(|guard| {
                (
                    guard.id,
                    guard.pos,
                    guard.kind.threat_bonus(guard.star_level),
                )
            })
            .collect();
        let carriage_pos = self.carriage.pos;
        let cargo_protection = self.cargo_protection;
        let hub_damage = self.hub_damage;
        let ward_radius = self.ward_radius;
        let mut pending_guard_damage = Vec::new();
        let mut pending_carriage_damage = Vec::new();
        let mut pending_summons = Vec::new();

        for enemy in &mut self.enemies {
            if !enemy.is_active() {
                continue;
            }

            enemy.cooldown = (enemy.cooldown - dt).max(0.0);
            enemy.special_timer = (enemy.special_timer - dt).max(0.0);
            enemy.slow_timer = (enemy.slow_timer - dt).max(0.0);
            enemy.hit_flash = (enemy.hit_flash - dt).max(0.0);

            // Warding Lantern: keep nearby enemies slowed while they linger.
            if ward_radius > 0.0 && enemy.pos.distance(carriage_pos) < ward_radius {
                enemy.slow_timer = enemy.slow_timer.max(0.4);
            }
            // Spiked Hubs: bleed enemies that press against the carriage.
            if hub_damage > 0.0 && enemy.pos.distance(carriage_pos) < 60.0 {
                enemy.health -= hub_damage * dt;
                enemy.hit_flash = 0.08;
                if !enemy.is_active() {
                    continue;
                }
            }

            if enemy.kind == EnemyKind::Necromancer && enemy.special_timer <= 0.0 {
                pending_summons.push(enemy.pos + vec2(0.0, 36.0));
                enemy.special_timer = 4.6;
            }

            let guard_target = guards
                .iter()
                .filter(|(_, pos, threat)| {
                    pos.distance(enemy.pos) < enemy.guard_aggro_range() + *threat
                })
                .min_by(|(_, a, _), (_, b, _)| {
                    a.distance(enemy.pos)
                        .partial_cmp(&b.distance(enemy.pos))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied();
            let (target_pos, target_guard) = guard_target
                .map(|(id, pos, _)| (pos, Some(id)))
                .unwrap_or((carriage_pos, None));

            let base_speed = if enemy.slow_timer > 0.0 {
                enemy.speed * 0.48
            } else {
                enemy.speed
            };

            // A thief that already grabbed cargo sprints for the top edge and
            // ignores everything until it either escapes or is cut down.
            if enemy.retreating {
                enemy.pos = move_towards(enemy.pos, vec2(enemy.pos.x, -160.0), base_speed * dt);
                continue;
            }

            let dist = enemy.pos.distance(target_pos);
            let in_attack_range = match enemy.kind.kite_min_range() {
                // Skirmishers back away when a target crowds them.
                Some(min_range) if dist < min_range => {
                    let away = clamp_to_field(enemy.pos + (enemy.pos - target_pos));
                    enemy.pos = move_towards(enemy.pos, away, base_speed * dt);
                    false
                }
                _ => {
                    if dist > enemy.attack_range {
                        // Chargers commit to a speed burst on the final approach.
                        let charge = if dist < 150.0 {
                            enemy.kind.charge_multiplier()
                        } else {
                            1.0
                        };
                        enemy.pos = move_towards(enemy.pos, target_pos, base_speed * charge * dt);
                        false
                    } else {
                        true
                    }
                }
            };

            if in_attack_range && enemy.cooldown <= 0.0 {
                enemy.cooldown = enemy.attack_cooldown;
                if let Some(guard_id) = target_guard {
                    pending_guard_damage.push((guard_id, enemy.damage));
                } else {
                    let cargo_loss = if enemy.kind.steals_and_flees() {
                        enemy.carried_cargo += 6.0 * (1.0 - cargo_protection);
                        enemy.retreating = true;
                        6.0
                    } else {
                        0.0
                    };
                    pending_carriage_damage.push((
                        enemy.damage,
                        cargo_loss,
                        enemy.kind.attack_label(),
                    ));
                }
            }
        }

        for (guard_id, damage) in pending_guard_damage {
            self.damage_guard(guard_id, damage);
        }
        for (damage, cargo_loss, label) in pending_carriage_damage {
            self.damage_carriage(damage, cargo_loss, label);
        }
        for pos in pending_summons {
            if self.enemies.len() >= MAX_LIVE_ENEMIES {
                break;
            }
            self.enemies.push(Enemy::new(
                self.next_enemy_id,
                EnemyKind::Skeleton,
                clamp_to_field(pos),
                1.0,
            ));
            self.next_enemy_id += 1;
            self.alert.set("Skeletons raised");
        }
    }

    pub(super) fn update_shots(&mut self, dt: f32) {
        for shot in &mut self.shots {
            shot.timer -= dt;
        }
        self.shots.retain(|shot| shot.timer > 0.0);
    }

    pub(super) fn handle_hazard_collisions(&mut self, dt: f32) {
        let carriage_rect = self.carriage.rect();
        let mut impacts = Vec::new();

        for hazard in &mut self.hazards {
            if !hazard.active {
                continue;
            }

            match hazard.kind {
                HazardKind::Mud => {
                    if hazard.pos.distance(self.carriage.pos) <= hazard.radius + 34.0 {
                        self.carriage.slow_timer = 1.25;
                        if !hazard.triggered {
                            hazard.triggered = true;
                            impacts.push((0.0, 1.5, "Mud slowed the wheels"));
                        }
                    }
                }
                HazardKind::FallenTree => {
                    if hazard.rect().overlaps(&carriage_rect) {
                        hazard.active = false;
                        impacts.push((17.0, 5.0, "Fallen tree impact"));
                    }
                }
                HazardKind::Rocks => {
                    if hazard.pos.distance(self.carriage.pos) <= hazard.radius + 36.0 {
                        hazard.active = false;
                        impacts.push((12.0, 3.0, "Rock strike"));
                    }
                }
                HazardKind::FirePatch => {
                    if hazard.pos.distance(self.carriage.pos) <= hazard.radius + 34.0 {
                        let label = if hazard.triggered { "" } else { "Fire patch" };
                        hazard.triggered = true;
                        impacts.push((8.5 * dt, 2.4 * dt, label));
                    }
                }
                HazardKind::RiverFord => {
                    // A long, strong slow — no real damage, just lost time and a
                    // small cargo jostle on entry.
                    if hazard.pos.distance(self.carriage.pos) <= hazard.radius + 40.0 {
                        self.carriage.slow_timer = 2.1;
                        if !hazard.triggered {
                            hazard.triggered = true;
                            impacts.push((0.0, 2.0, "River ford slowed the wheels"));
                        }
                    }
                }
            }
        }

        for (damage, cargo_loss, label) in impacts {
            self.damage_carriage(damage, cargo_loss, label);
        }
    }

    pub(super) fn cleanup_entities(&mut self) {
        self.enemies_defeated += self
            .enemies
            .iter()
            .filter(|enemy| enemy.health <= 0.0)
            .count() as u32;

        // Killing a fleeing thief returns the cargo it was carrying off.
        let recovered: f32 = self
            .enemies
            .iter()
            .filter(|enemy| enemy.health <= 0.0 && enemy.carried_cargo > 0.0)
            .map(|enemy| enemy.carried_cargo)
            .sum();
        if recovered > 0.0 {
            self.carriage.cargo = (self.carriage.cargo + recovered).min(self.carriage.max_cargo);
            self.cargo_lost = (self.cargo_lost - recovered).max(0.0);
            self.alert.set("Cargo recovered");
        }

        // Death bursts (juice): so a slain enemy scatters instead of vanishing.
        let deaths: Vec<(Vec2, Color)> = self
            .enemies
            .iter()
            .filter(|enemy| enemy.health <= 0.0)
            .map(|enemy| (enemy.pos, death_color(enemy.kind)))
            .collect();
        for (pos, color) in deaths {
            self.spawn_burst(pos, color);
        }

        self.enemies.retain(|enemy| {
            enemy.health > 0.0
                && enemy.pos.y > PLAY_TOP - 150.0
                && enemy.pos.y < PLAY_BOTTOM + 120.0
                && enemy.pos.x > -170.0
                && enemy.pos.x < PLAY_WIDTH + 170.0
        });
        self.hazards
            .retain(|hazard| hazard.pos.y < PLAY_BOTTOM + 95.0);
    }

    fn spawn_burst(&mut self, pos: Vec2, color: Color) {
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::TAU / 6.0 + pos.x * 0.03;
            let (sin, cos) = angle.sin_cos();
            let speed = 55.0 + i as f32 * 9.0;
            self.particles.push(Particle::new(
                pos,
                vec2(cos * speed, sin * speed),
                color,
                4.5,
            ));
        }
    }

    fn apply_guard_hit(&mut self, hit: PendingGuardHit) {
        let mut damage = hit.damage;
        if hit.kind == GuardKind::CrossbowGuard
            && hit.stars >= 2
            && matches!(
                hit.enemy_kind,
                EnemyKind::Skeleton | EnemyKind::Necromancer | EnemyKind::ArmoredBandit
            )
        {
            damage *= 1.35;
        }

        let Some(primary_pos) = self.damage_enemy(hit.enemy_id, damage) else {
            return;
        };

        if hit.kind.is_ranged() {
            self.shots
                .push(Shot::new(hit.origin, hit.target, hit.kind.shot_color()));
        }

        match hit.kind {
            GuardKind::Swordsman if hit.stars >= 3 => {
                if let Some(extra_id) = self.nearby_enemy(hit.enemy_id, primary_pos, 48.0) {
                    self.damage_enemy(extra_id, damage * 0.45);
                }
            }
            GuardKind::Archer if hit.stars >= 3 => {
                if let Some(extra_id) = self.nearby_enemy(hit.enemy_id, primary_pos, 78.0) {
                    self.damage_enemy(extra_id, damage * 0.65);
                }
            }
            GuardKind::CrossbowGuard if hit.stars >= 3 => {
                if let Some(enemy) = self
                    .enemies
                    .iter_mut()
                    .find(|enemy| enemy.id == hit.enemy_id)
                {
                    enemy.slow_timer = 1.25;
                }
            }
            GuardKind::Mage => {
                if hit.stars >= 2 {
                    let splash_ids: Vec<u32> = self
                        .enemies
                        .iter()
                        .filter(|enemy| {
                            enemy.id != hit.enemy_id
                                && enemy.is_active()
                                && enemy.pos.distance(primary_pos) < 62.0
                        })
                        .map(|enemy| enemy.id)
                        .collect();
                    for splash_id in splash_ids {
                        self.damage_enemy(splash_id, damage * 0.38);
                    }
                }
                if hit.stars >= 3 {
                    self.heal_weak_guard(4.0 + damage * 0.08);
                }
            }
            _ => {}
        }
    }

    fn damage_enemy(&mut self, enemy_id: u32, damage: f32) -> Option<Vec2> {
        let pos = {
            let enemy = self.enemies.iter_mut().find(|enemy| enemy.id == enemy_id)?;
            enemy.health -= damage;
            enemy.hit_flash = 0.12;
            enemy.pos
        };
        self.float_texts.push(FloatText::new(
            vec2(pos.x, pos.y - 22.0),
            format!("{:.0}", damage.max(1.0)),
            Color::new(0.98, 0.90, 0.52, 1.0),
        ));
        Some(pos)
    }

    fn nearby_enemy(&self, excluded_id: u32, pos: Vec2, radius: f32) -> Option<u32> {
        self.enemies
            .iter()
            .filter(|enemy| enemy.id != excluded_id && enemy.is_active())
            .filter(|enemy| enemy.pos.distance(pos) <= radius)
            .min_by(|a, b| {
                a.pos
                    .distance(pos)
                    .partial_cmp(&b.pos.distance(pos))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|enemy| enemy.id)
    }

    fn heal_weak_guard(&mut self, amount: f32) {
        if let Some(guard) = self
            .guards
            .iter_mut()
            .filter(|guard| guard.is_active() && guard.health < guard.max_health)
            .min_by(|a, b| {
                (a.health / a.max_health)
                    .partial_cmp(&(b.health / b.max_health))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            guard.health = (guard.health + amount).min(guard.max_health);
        }
    }

    fn damage_guard(&mut self, guard_id: u32, damage: f32) {
        if let Some(guard) = self.guards.iter_mut().find(|guard| guard.id == guard_id) {
            let final_damage = (damage - guard.armor).max(damage * 0.35);
            guard.health = (guard.health - final_damage).max(0.0);
            guard.hit_flash = 0.18;
            self.guard_damage_taken += final_damage;
            if self.mission_kind == MissionKind::RefugeeEscort {
                self.special_meter = (self.special_meter - final_damage * 0.08).max(0.0);
            }
            if guard.health <= 0.0 {
                guard.order = GuardOrder::Hold;
                guard.mounted_slot = None;
                if matches!(
                    self.mission_kind,
                    MissionKind::RefugeeEscort | MissionKind::SiegeSupplyRun
                ) {
                    self.special_meter = (self.special_meter - 7.0).max(0.0);
                }
                self.alert.set("Guard down");
            }
        }
    }

    fn damage_carriage(&mut self, damage: f32, cargo_loss: f32, label: &str) {
        let shield_wall = self.guards.iter().any(|guard| {
            guard.kind == GuardKind::ShieldGuard
                && guard.star_level >= 3
                && guard.is_active()
                && guard.pos.distance(self.carriage.pos) < 126.0
        });
        let final_damage = if damage > 0.0 {
            (damage - self.armor_reduction).max(damage * 0.35)
        } else {
            0.0
        } * if shield_wall { 0.72 } else { 1.0 };
        let final_cargo_loss = cargo_loss * (1.0 - self.cargo_protection);

        self.carriage.health = (self.carriage.health - final_damage).max(0.0);
        self.carriage.cargo = (self.carriage.cargo - final_cargo_loss).max(0.0);
        if final_damage >= 1.0 {
            self.float_texts.push(FloatText::new(
                vec2(self.carriage.pos.x, self.carriage.pos.y - 44.0),
                format!("-{:.0}", final_damage),
                Color::new(0.98, 0.42, 0.34, 1.0),
            ));
        }
        match self.mission_kind {
            MissionKind::PrincessEscort => {
                self.special_meter =
                    (self.special_meter - final_damage * 0.42 - final_cargo_loss * 0.35).max(0.0);
            }
            MissionKind::MedicineRun => {
                self.special_meter =
                    (self.special_meter - final_damage * 0.32 - final_cargo_loss * 0.72).max(0.0);
            }
            MissionKind::GoldShipment => {
                let theft_pressure = if label.contains("stole") { 2.5 } else { 0.0 };
                self.special_meter = (self.special_meter
                    - final_damage * 0.08
                    - final_cargo_loss * 1.18
                    - theft_pressure)
                    .max(0.0);
            }
            MissionKind::MonsterEggTransport => {
                self.special_meter =
                    (self.special_meter - final_damage * 0.56 - final_cargo_loss * 0.42).max(0.0);
            }
            MissionKind::RefugeeEscort => {
                self.special_meter =
                    (self.special_meter - final_damage * 0.28 - final_cargo_loss * 0.38).max(0.0);
            }
            MissionKind::RoyalBanquetSupplies => {
                let heat_pressure = if label.contains("Fire") { 3.0 } else { 0.0 };
                self.special_meter = (self.special_meter
                    - final_damage * 0.18
                    - final_cargo_loss * 0.86
                    - heat_pressure)
                    .max(0.0);
            }
            MissionKind::SiegeSupplyRun => {
                self.special_meter =
                    (self.special_meter - final_damage * 0.20 - final_cargo_loss * 0.32).max(0.0);
            }
            MissionKind::CargoTransfer
            | MissionKind::PrisonerEscort
            | MissionKind::TimeDelivery => {}
        }
        self.damage_taken += final_damage;
        self.cargo_lost += final_cargo_loss;
        self.carriage.hit_flash = 0.22;
        if !label.is_empty() {
            self.alert.set(label);
        }
    }

    fn escort_positions(&self) -> Vec<Vec2> {
        self.guards
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let side = if index % 2 == 0 { -1.0 } else { 1.0 };
                let row = (index / 2) as f32;
                vec2(
                    self.carriage.pos.x + side * (68.0 + row * 18.0),
                    self.carriage.pos.y + 32.0 + row * 34.0,
                )
            })
            .collect()
    }
}

fn nearest_enemy_in_range(
    enemies: &[(u32, Vec2, f32, EnemyKind)],
    origin: Vec2,
    range: f32,
) -> Option<(u32, Vec2, f32, EnemyKind)> {
    enemies
        .iter()
        .filter(|(_, pos, radius, _)| pos.distance(origin) <= range + *radius)
        .min_by(|(_, a, _, _), (_, b, _, _)| {
            a.distance(origin)
                .partial_cmp(&b.distance(origin))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

fn melee_bonus(kind: GuardKind, stars: u8, enemy_kind: EnemyKind) -> f32 {
    if kind == GuardKind::Spearman && enemy_kind == EnemyKind::Wolf {
        if stars >= 3 {
            1.8
        } else {
            1.35
        }
    } else {
        1.0
    }
}

fn auto_attack_range(guard: &Guard) -> f32 {
    if guard.kind.is_ranged() {
        guard.range
    } else {
        guard.range + 24.0
    }
}

fn guard_hit_damage(guard: &Guard, enemy_kind: EnemyKind) -> f32 {
    let bonus = if guard.kind.is_melee() {
        melee_bonus(guard.kind, guard.star_level, enemy_kind)
    } else {
        1.0
    };
    guard.attack * bonus
}

/// Tint of an enemy's death burst, roughly matching its sprite.
fn death_color(kind: EnemyKind) -> Color {
    match kind {
        EnemyKind::Wolf | EnemyKind::AlphaWolf => Color::new(0.55, 0.55, 0.58, 1.0),
        EnemyKind::Bandit | EnemyKind::ArmoredBandit => Color::new(0.72, 0.30, 0.22, 1.0),
        EnemyKind::BanditArcher => Color::new(0.66, 0.42, 0.24, 1.0),
        EnemyKind::Skeleton => Color::new(0.86, 0.87, 0.83, 1.0),
        EnemyKind::Necromancer => Color::new(0.56, 0.30, 0.68, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MissionDef;
    use crate::state::CampaignState;

    fn test_run() -> MissionRun {
        let config = crate::data::GameConfig {
            game_name: "carriage_run".to_owned(),
            display_name: "Carriage Run".to_owned(),
            save_slot: "campaign".to_owned(),
            version: "0.1.0".to_owned(),
            starting_gold: 100,
        };
        let campaign = CampaignState::new(&config, Some("muddy_road"));
        let mission = MissionDef {
            id: "muddy_road".to_owned(),
            name: "The Muddy Road".to_owned(),
            order: 1,
            mission_type: "cargo_transfer".to_owned(),
            route: "Forest Road".to_owned(),
            cargo: "Basic Supplies".to_owned(),
            objective: "Reach the village.".to_owned(),
            bonus_objective: "Keep cargo safe.".to_owned(),
            intro_text: String::new(),
            bonus: None,
            outro_text: String::new(),
            unlock_level: 1,
            distance: 500.0,
            difficulty: 1.0,
            base_reward: 100,
            enemy_mix: vec!["wolf".to_owned()],
            hazard_mix: Vec::new(),
            route_choices: Vec::new(),
            prerequisite_missions: Vec::new(),
            unlock_any_missions: Vec::new(),
            time_limit: None,
        };
        MissionRun::new(&mission, &campaign)
    }

    #[test]
    fn alpha_wolf_outclasses_a_common_wolf() {
        let wolf = Enemy::new(1, EnemyKind::Wolf, vec2(0.0, 0.0), 1.0);
        let alpha = Enemy::new(2, EnemyKind::AlphaWolf, vec2(0.0, 0.0), 1.0);
        assert!(alpha.max_health > wolf.max_health);
        assert!(alpha.damage > wolf.damage);
        assert!(alpha.speed > wolf.speed);
    }

    #[test]
    fn live_enemies_are_hard_capped() {
        let mut run = test_run();
        // Far more spawn attempts than the cap; count must never exceed it.
        for _ in 0..(MAX_LIVE_ENEMIES * 4) {
            run.spawn_enemy();
        }
        assert_eq!(run.enemies.len(), MAX_LIVE_ENEMIES);
    }

    #[test]
    fn roaming_melee_guard_auto_hits_nearby_enemy() {
        let mut run = test_run();
        let guard_pos = run
            .guards
            .iter()
            .find(|guard| guard.kind == GuardKind::Swordsman)
            .unwrap()
            .pos;
        run.enemies.push(Enemy::new(
            99,
            EnemyKind::Wolf,
            guard_pos + vec2(28.0, 0.0),
            1.0,
        ));
        let before = run.enemies[0].health;

        run.update_guard_orders(0.2);

        assert!(run.enemies[0].health < before);
        assert!(run.guards.iter().any(|guard| guard.attack_flash > 0.0));
    }

    #[test]
    fn roaming_guard_advances_on_distant_enemy_within_leash() {
        let mut run = test_run();
        // An enemy inside the leash but well beyond weapon reach should be
        // chased, not ignored.
        let target = run.carriage.pos + vec2(0.0, -190.0);
        run.enemies
            .push(Enemy::new(77, EnemyKind::Wolf, target, 1.0));
        let guard_id = run
            .guards
            .iter()
            .find(|guard| guard.kind == GuardKind::Swordsman)
            .unwrap()
            .id;
        let before = run
            .guards
            .iter()
            .find(|guard| guard.id == guard_id)
            .unwrap()
            .pos
            .distance(target);

        run.update_guard_orders(0.2);

        let after = run
            .guards
            .iter()
            .find(|guard| guard.id == guard_id)
            .unwrap()
            .pos
            .distance(target);
        assert!(after < before, "roaming guard should close on the threat");
    }

    #[test]
    fn spiked_hubs_wound_adjacent_enemies() {
        let mut run = test_run();
        run.hub_damage = 20.0;
        run.enemies
            .push(Enemy::new(42, EnemyKind::Wolf, run.carriage.pos, 1.0));
        let before = run.enemies[0].health;

        run.update_enemies(0.5);

        assert!(run.enemies[0].health < before);
    }

    #[test]
    fn killing_fleeing_thief_recovers_cargo() {
        let mut run = test_run();
        run.guards.clear(); // isolate the carriage so the bandit targets it
        let full_cargo = run.carriage.cargo;
        run.enemies
            .push(Enemy::new(55, EnemyKind::Bandit, run.carriage.pos, 1.0));

        // Advance until the bandit steals and turns to flee.
        for _ in 0..40 {
            run.update_enemies(0.2);
            if run.enemies.iter().any(|enemy| enemy.retreating) {
                break;
            }
        }
        let thief = &run.enemies[0];
        assert!(thief.retreating, "bandit should flee after stealing");
        assert!(thief.carried_cargo > 0.0);
        assert!(
            run.carriage.cargo < full_cargo,
            "cargo should drop on theft"
        );

        // Cutting it down before it escapes returns the stolen cargo.
        run.enemies[0].health = 0.0;
        run.cleanup_entities();
        assert!((run.carriage.cargo - full_cargo).abs() < 0.001);
    }
}
