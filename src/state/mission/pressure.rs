//! Per-mission-type "special meter" pressure updates (security, comfort,
//! freshness, momentum, and so on). Kept separate from combat resolution.

use super::*;
use macroquad::prelude::*;

impl MissionRun {
    pub(super) fn update_mission_pressure(&mut self, dt: f32) {
        match self.mission_kind {
            MissionKind::CargoTransfer | MissionKind::TimeDelivery => {}
            MissionKind::PrisonerEscort => {
                let nearby_bandits = self
                    .enemies
                    .iter()
                    .filter(|enemy| {
                        matches!(enemy.kind, EnemyKind::Bandit | EnemyKind::BanditArcher)
                            && enemy.pos.distance(self.carriage.pos) < 145.0
                    })
                    .count() as f32;
                let rough_road_relief = if self.carriage.slow_timer > 0.0 {
                    2.4
                } else {
                    0.0
                };
                self.special_meter += dt * (1.1 + nearby_bandits * 2.1 - rough_road_relief);
                self.special_meter = self.special_meter.clamp(0.0, 100.0);
            }
            MissionKind::PrincessEscort => {
                let road_discomfort = if self.carriage.slow_timer > 0.0 {
                    0.7
                } else {
                    0.12
                };
                self.special_meter = (self.special_meter - dt * road_discomfort).clamp(0.0, 100.0);
            }
            MissionKind::MedicineRun => {
                let rough_decay = if self.carriage.slow_timer > 0.0 {
                    0.78
                } else {
                    0.18
                };
                self.special_meter = (self.special_meter - dt * rough_decay).clamp(0.0, 100.0);
            }
            MissionKind::GoldShipment => {
                let nearby_bandits = self
                    .enemies
                    .iter()
                    .filter(|enemy| {
                        matches!(enemy.kind, EnemyKind::Bandit | EnemyKind::BanditArcher)
                            && enemy.pos.distance(self.carriage.pos) < 150.0
                    })
                    .count() as f32;
                self.special_meter =
                    (self.special_meter - dt * nearby_bandits * 0.85).clamp(0.0, 100.0);
            }
            MissionKind::MonsterEggTransport => {
                // The stability meter is a bomb timer, not a fail meter: rough
                // road jostles the egg, and if stability runs out it HATCHES
                // mid-route and the brood swarms (see `hatch_egg`).
                if self.egg_hatched {
                    return;
                }
                let rough_decay = if self.carriage.slow_timer > 0.0 {
                    0.45
                } else {
                    0.06
                };
                self.special_meter = (self.special_meter - dt * rough_decay).clamp(0.0, 100.0);
                if self.special_meter <= 35.0 && !self.egg_cracked {
                    self.egg_cracked = true;
                    self.alert.set("The egg is cracking — keep it steady!");
                }
                if self.special_meter <= 0.0 {
                    self.hatch_egg();
                }
            }
            MissionKind::RefugeeEscort => {
                let nearby_threats = self
                    .enemies
                    .iter()
                    .filter(|enemy| enemy.pos.distance(self.carriage.pos) < 170.0)
                    .count() as f32;
                self.special_meter =
                    (self.special_meter - dt * nearby_threats * 0.42).clamp(0.0, 100.0);
            }
            MissionKind::RoyalBanquetSupplies => {
                let heat_decay = self
                    .hazards
                    .iter()
                    .filter(|hazard| hazard.kind == HazardKind::FirePatch && hazard.triggered)
                    .count() as f32
                    * 0.18;
                let road_decay = if self.carriage.slow_timer > 0.0 {
                    0.42
                } else {
                    0.15
                };
                self.special_meter =
                    (self.special_meter - dt * (road_decay + heat_decay)).clamp(0.0, 100.0);
            }
            MissionKind::SiegeSupplyRun => {
                // Momentum drains under an assault but recovers in the calm
                // between mega-waves, matching the inverted spawn rhythm.
                let nearby = self
                    .enemies
                    .iter()
                    .filter(|enemy| enemy.pos.distance(self.carriage.pos) < 190.0)
                    .count();
                let delta = if nearby == 0 && self.carriage.slow_timer <= 0.0 {
                    0.55
                } else {
                    let road_drag = if self.carriage.slow_timer > 0.0 {
                        0.34
                    } else {
                        0.10
                    };
                    -(road_drag + nearby as f32 * 0.28)
                };
                self.special_meter = (self.special_meter + dt * delta).clamp(0.0, 100.0);
            }
        }
    }

    /// The monster egg hatches mid-route: a brood erupts around the carriage and
    /// swarms it. Fires once when stability is spent (see the pressure update).
    fn hatch_egg(&mut self) {
        self.egg_hatched = true;
        self.special_meter = 0.0;
        self.alert.set("The egg HATCHES — the brood swarms!");
        let count = 4 + (self.difficulty as u32).min(3);
        for i in 0..count {
            if self.enemies.len() >= MAX_LIVE_ENEMIES {
                break;
            }
            let angle = i as f32 / count as f32 * std::f32::consts::TAU;
            let pos = self.carriage.pos + vec2(angle.cos(), angle.sin()) * 120.0;
            // Alternate fast wolves and skeletal hatchlings; a touch tougher than
            // a normal spawn so the hatch is a real punishment.
            let kind = if i % 2 == 0 {
                EnemyKind::Wolf
            } else {
                EnemyKind::Skeleton
            };
            self.enemies.push(Enemy::new(
                self.next_enemy_id,
                kind,
                pos,
                self.difficulty * 1.15,
            ));
            self.next_enemy_id += 1;
        }
    }
}
