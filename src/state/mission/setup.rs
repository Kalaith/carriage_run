//! Construction policy for a fresh mission run.

use super::{
    carriage_slot_pos, CampaignState, MissionKind, MissionRunContext, GENEROUS_TIMER_BONUS,
};
use crate::data::MissionDef;
use crate::state::entities::{Guard, CARRIAGE_Y, ROAD_CENTER, ROAD_LEFT};
use crate::state::CarriageEquipment;
use macroquad::prelude::*;

/// Resolved values copied into `MissionRun` when a route starts. Keeping route
/// and loadout resolution here leaves the parent model focused on simulation
/// and makes setup policy independently reviewable.
pub(super) struct MissionSetup {
    pub mission_kind: MissionKind,
    pub route_name: String,
    pub max_health: f32,
    pub cargo_max: f32,
    pub guards: Vec<Guard>,
    pub ranged_slots: usize,
    pub enemy_mix: Vec<String>,
    pub hazard_mix: Vec<String>,
    pub distance: f32,
    pub difficulty: f32,
    pub base_reward: i64,
    pub time_limit: Option<f32>,
    pub seed: u64,
    pub wave_pace: f32,
    pub chassis_speed_mult: f32,
    pub armor_reduction: f32,
    pub cargo_protection: f32,
    pub wheel_bonus: f32,
    pub repair_heal: f32,
    pub hub_damage: f32,
    pub ward_radius: f32,
}

impl MissionSetup {
    pub(super) fn resolve(
        mission: &MissionDef,
        campaign: &CampaignState,
        context: MissionRunContext,
    ) -> Self {
        let assist_health = if campaign.sturdy_carriage { 1.25 } else { 1.0 };
        let wave_pace = if campaign.slower_waves { 1.5 } else { 1.0 };
        let max_health = (100.0 + campaign.armor_level as f32 * 26.0)
            * campaign.chassis_health_mult
            * campaign.frame_health_mult
            * assist_health;
        let cargo_max = (100.0 + campaign.cargo_level as f32 * 6.0) * campaign.frame_cargo_mult;
        let route_choice = match context {
            MissionRunContext::Campaign => campaign.selected_route_choice(mission),
            MissionRunContext::Expedition { .. } => None,
        };
        let route_choice_id = route_choice
            .map(|choice| choice.id.clone())
            .unwrap_or_default();
        let route_seed = route_choice_id.bytes().fold(0_u64, |seed, byte| {
            seed.wrapping_mul(37).wrapping_add(byte as u64)
        });
        let seed = match context {
            MissionRunContext::Campaign => {
                mission.order as u64 * 10_007
                    + campaign
                        .records
                        .get(&mission.id)
                        .map(|record| record.completions as u64)
                        .unwrap_or(0)
                    + route_seed
            }
            MissionRunContext::Expedition { seed } => seed,
        };

        let mission_kind = MissionKind::from_id(&mission.mission_type);
        let mut enemy_mix = mission.enemy_mix.clone();
        let mut hazard_mix = mission.hazard_mix.clone();
        if let Some(choice) = route_choice {
            enemy_mix.extend(choice.enemy_add.iter().cloned());
            hazard_mix.extend(choice.hazard_add.iter().cloned());
        }
        let distance = route_choice
            .map(|choice| mission.distance + choice.distance_delta)
            .unwrap_or(mission.distance)
            .max(420.0);
        let authored_content_scale = if mission.authored_act() > 1 {
            0.72
        } else {
            1.0
        };
        let difficulty = (route_choice
            .map(|choice| mission.difficulty + choice.difficulty_delta)
            .unwrap_or(mission.difficulty)
            .max(0.6)
            * campaign.difficulty_preset.difficulty_scale())
            * authored_content_scale
            * if mission_kind == MissionKind::SiegeSupplyRun {
                0.60
            } else {
                1.0
            };
        let difficulty = difficulty.max(0.5);
        let base_reward = route_choice
            .map(|choice| mission.base_reward + choice.reward_delta)
            .unwrap_or(mission.base_reward)
            .max(0);
        let time_limit = mission.time_limit.map(|limit| {
            let base = route_choice
                .map(|choice| limit + choice.time_limit_delta)
                .unwrap_or(limit)
                .max(30.0);
            base + if campaign.generous_timers {
                GENEROUS_TIMER_BONUS
            } else {
                0.0
            }
        });

        let mut guards = Vec::new();
        for (index, kind) in campaign.selected_melee_kinds().into_iter().enumerate() {
            if !campaign.is_guard_available(kind) {
                continue;
            }
            let side = if index % 2 == 0 { -1.0 } else { 1.0 };
            guards.push(Guard::new(
                index as u32 + 1,
                kind,
                vec2(
                    ROAD_LEFT + 235.0 + side * index as f32 * 52.0,
                    CARRIAGE_Y + 34.0,
                ),
                campaign.guard_level,
                campaign.archer_level,
                campaign.guard_star_level(kind),
                None,
            ));
        }

        let ranged_slots = campaign.ranged_slot_count();
        for (index, kind) in campaign.selected_ranged_kinds().into_iter().enumerate() {
            if !campaign.is_guard_available(kind) {
                continue;
            }
            guards.push(Guard::new(
                guards.len() as u32 + 1,
                kind,
                carriage_slot_pos(ROAD_CENTER, index, ranged_slots),
                campaign.guard_level,
                campaign.archer_level,
                campaign.guard_star_level(kind),
                Some(index),
            ));
        }
        for guard in &mut guards {
            guard.specialized = campaign.guard_specialization(guard.kind).is_some();
        }

        let armor_equipped = campaign.is_equipment_equipped(CarriageEquipment::IronPlating);
        let wheels_equipped = campaign.is_equipment_equipped(CarriageEquipment::ReinforcedWheels);
        let straps_equipped = campaign.is_equipment_equipped(CarriageEquipment::CargoStraps);
        let repair_equipped = campaign.is_equipment_equipped(CarriageEquipment::RepairKit);
        let hubs_equipped = campaign.is_equipment_equipped(CarriageEquipment::SpikedHubs);
        let lantern_equipped = campaign.is_equipment_equipped(CarriageEquipment::WardingLantern);

        Self {
            mission_kind,
            route_name: route_choice
                .map(|choice| choice.name.clone())
                .unwrap_or_else(|| mission.route.clone()),
            max_health,
            cargo_max,
            guards,
            ranged_slots,
            enemy_mix,
            hazard_mix,
            distance,
            difficulty,
            base_reward,
            time_limit,
            seed,
            wave_pace,
            chassis_speed_mult: campaign.chassis_speed_mult * campaign.frame_speed_mult,
            armor_reduction: if armor_equipped {
                campaign.armor_level as f32 * 1.8
            } else {
                campaign.armor_level as f32 * 0.45
            },
            cargo_protection: if straps_equipped {
                (campaign.cargo_level as f32 * 0.12).min(0.42)
            } else {
                0.0
            },
            wheel_bonus: if wheels_equipped {
                campaign.wheel_level as f32 * 1.5
            } else {
                0.0
            },
            repair_heal: if repair_equipped {
                campaign.repair_level as f32 * 22.0
            } else {
                0.0
            },
            hub_damage: if hubs_equipped {
                8.0 + campaign.hubs_level as f32 * 7.0
            } else {
                0.0
            },
            ward_radius: if lantern_equipped {
                86.0 + campaign.lantern_level as f32 * 20.0
            } else {
                0.0
            },
        }
    }
}
