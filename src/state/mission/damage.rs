//! Applying damage to enemies, guards, and the carriage, plus guard-hit perks.

use super::*;

/// A guard's landed attack, resolved after the guard loop releases its borrow
/// on `self.guards`.
pub(super) struct PendingGuardHit {
    pub(super) kind: GuardKind,
    pub(super) stars: u8,
    pub(super) enemy_id: u32,
    pub(super) enemy_kind: EnemyKind,
    pub(super) damage: f32,
    pub(super) origin: Vec2,
    pub(super) target: Vec2,
}

impl MissionRun {
    pub(super) fn apply_guard_hit(&mut self, hit: PendingGuardHit) {
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
            enemy.hit_flash = Timer::new(0.12);
            enemy.pos
        };
        self.float_texts.spawn(
            format!("{:.0}", damage.max(1.0)),
            vec2(pos.x, pos.y - 22.0),
            Color::new(0.98, 0.90, 0.52, 1.0),
        );
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

    pub(super) fn damage_guard(&mut self, guard_id: u32, damage: f32) {
        if let Some(guard) = self.guards.iter_mut().find(|guard| guard.id == guard_id) {
            let final_damage = (damage - guard.armor).max(damage * 0.35);
            guard.health = (guard.health - final_damage).max(0.0);
            guard.hit_flash = Timer::new(0.18);
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

    pub(super) fn damage_carriage(&mut self, damage: f32, cargo_loss: f32, label: &str) {
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
            self.float_texts.spawn(
                format!("-{:.0}", final_damage),
                vec2(self.carriage.pos.x, self.carriage.pos.y - 44.0),
                Color::new(0.98, 0.42, 0.34, 1.0),
            );
        }
        self.apply_special_meter_pressure(final_damage, final_cargo_loss, label);
        self.damage_taken += final_damage;
        self.cargo_lost += final_cargo_loss;
        self.carriage.hit_flash = Timer::new(0.22);
        if !label.is_empty() {
            self.alert.set(label);
        }
    }

    /// Each mission kind converts a hit into its own bespoke objective pressure:
    /// a princess loses composure, medicine loses potency, gold gets stolen.
    fn apply_special_meter_pressure(&mut self, damage: f32, cargo_loss: f32, label: &str) {
        let drain = match self.mission_kind {
            MissionKind::PrincessEscort => damage * 0.42 + cargo_loss * 0.35,
            MissionKind::MedicineRun => damage * 0.32 + cargo_loss * 0.72,
            MissionKind::GoldShipment => {
                let theft_pressure = if label.contains("stole") { 2.5 } else { 0.0 };
                damage * 0.08 + cargo_loss * 1.18 + theft_pressure
            }
            MissionKind::MonsterEggTransport => damage * 0.56 + cargo_loss * 0.42,
            MissionKind::RefugeeEscort => damage * 0.28 + cargo_loss * 0.38,
            MissionKind::RoyalBanquetSupplies => {
                let heat_pressure = if label.contains("Fire") { 3.0 } else { 0.0 };
                damage * 0.18 + cargo_loss * 0.86 + heat_pressure
            }
            MissionKind::SiegeSupplyRun => damage * 0.20 + cargo_loss * 0.32,
            MissionKind::CargoTransfer
            | MissionKind::PrisonerEscort
            | MissionKind::TimeDelivery => return,
        };
        self.special_meter = (self.special_meter - drain).max(0.0);
    }
}
