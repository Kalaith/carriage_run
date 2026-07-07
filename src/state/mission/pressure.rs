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
                let rough_decay = if self.carriage.slow_timer > 0.0 {
                    0.45
                } else {
                    0.06
                };
                self.special_meter = (self.special_meter - dt * rough_decay).clamp(0.0, 100.0);
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
                let enemy_pressure = self
                    .enemies
                    .iter()
                    .filter(|enemy| enemy.pos.distance(self.carriage.pos) < 190.0)
                    .count() as f32
                    * 0.28;
                let road_drag = if self.carriage.slow_timer > 0.0 {
                    0.34
                } else {
                    0.10
                };
                self.special_meter =
                    (self.special_meter - dt * (road_drag + enemy_pressure)).clamp(0.0, 100.0);
            }
        }
    }
}
