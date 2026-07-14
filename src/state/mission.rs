//! Active route simulation for Carriage Run missions.

mod combat;
mod flow;
mod pressure;
mod scoring;

use super::entities::*;
use super::{CampaignState, CarriageEquipment, CarriageVisual};
use crate::data::MissionDef;
use macroquad::prelude::*;
use macroquad_toolkit::fx::{FloatingTextLayer, ParticleSystem};
use macroquad_toolkit::rng::SeededRng;
use macroquad_toolkit::timing::Timer;

/// Extra seconds added to every timed mission when the "Generous Timers"
/// accessibility assist is on.
const GENEROUS_TIMER_BONUS: f32 = 15.0;

/// Bonus carriage health granted by spending one Reinforced Kit consumable.
const REINFORCED_KIT_HEALTH: f32 = 55.0;

/// Lifetime (seconds) of a floating combat number before it fades out.
const FLOAT_TEXT_LIFE: f32 = 0.7;

/// Hard ceiling on simultaneously live enemies. Well above what normal play
/// produces, so it never affects balance — it only backstops pathological
/// growth (e.g. necromancers raising skeletons faster than they die) that would
/// otherwise degrade performance in a long run.
pub(super) const MAX_LIVE_ENEMIES: usize = 48;

#[derive(Debug, Clone, Copy)]
pub struct MissionInput {
    pub mouse: Vec2,
    pub pressed: bool,
    pub down: bool,
    pub released: bool,
    pub repair_pressed: bool,
    pub play_rect: Rect,
    /// Keyboard drive state, injected so the sim stays headless-testable rather
    /// than reading `is_key_down` from inside `update`.
    pub steer_left: bool,
    pub steer_right: bool,
    pub boost: bool,
    pub brake: bool,
}

/// Per-frame keyboard drive state carried from `handle_input` into
/// `handle_keyboard` (which has the `dt` needed to apply it).
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct DriveKeys {
    pub left: bool,
    pub right: bool,
    pub boost: bool,
    pub brake: bool,
}

#[derive(Debug, Clone)]
pub struct MissionReport {
    pub mission_id: String,
    pub mission_name: String,
    pub route_name: String,
    pub success: bool,
    pub reason: String,
    pub stars: u8,
    pub score: i64,
    pub reward: i64,
    /// Gold lost to repairs and spoiled cargo when a run fails (0 on success).
    pub gold_penalty: i64,
    pub elapsed: f32,
    pub time_limit: Option<f32>,
    pub carriage_health_ratio: f32,
    pub cargo_ratio: f32,
    pub special_label: Option<String>,
    pub special_ratio: Option<f32>,
    pub enemies_defeated: u32,
    pub injured_guard_ids: Vec<String>,
    /// Whether the mission's bonus objective was achieved. `None` when the
    /// mission defines no structured bonus criteria.
    pub bonus_met: Option<bool>,
}

/// Drives enemy spawning as telegraphed bursts with breathing room between
/// them, rather than a constant trickle.
#[derive(Debug, Clone)]
pub(super) enum WavePhase {
    /// Quiet stretch; `timer` counts down to the next telegraph.
    Lull(f32),
    /// Warning shown; `timer` counts down to the burst.
    Telegraph(f32),
    /// Spawning a burst: `remaining` enemies left, `timer` to the next spawn.
    Active { remaining: u32, timer: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionKind {
    CargoTransfer,
    PrisonerEscort,
    PrincessEscort,
    MedicineRun,
    GoldShipment,
    MonsterEggTransport,
    RefugeeEscort,
    RoyalBanquetSupplies,
    SiegeSupplyRun,
    TimeDelivery,
}

impl MissionKind {
    fn from_id(id: &str) -> Self {
        match id {
            "prisoner_escort" => Self::PrisonerEscort,
            "princess_escort" => Self::PrincessEscort,
            "medicine_run" => Self::MedicineRun,
            "gold_shipment" => Self::GoldShipment,
            "monster_egg_transport" => Self::MonsterEggTransport,
            "refugee_escort" => Self::RefugeeEscort,
            "royal_banquet_supplies" => Self::RoyalBanquetSupplies,
            "siege_supply_run" => Self::SiegeSupplyRun,
            "time_delivery" => Self::TimeDelivery,
            _ => Self::CargoTransfer,
        }
    }

    pub fn label(self) -> Option<&'static str> {
        match self {
            Self::CargoTransfer => None,
            Self::PrisonerEscort => Some("Security"),
            Self::PrincessEscort => Some("Comfort"),
            Self::MedicineRun => Some("Potency"),
            Self::GoldShipment => Some("Gold"),
            Self::MonsterEggTransport => Some("Stability"),
            Self::RefugeeEscort => Some("Safety"),
            Self::RoyalBanquetSupplies => Some("Freshness"),
            Self::SiegeSupplyRun => Some("Momentum"),
            Self::TimeDelivery => Some("Deadline"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MissionRun {
    pub mission_id: String,
    pub mission_name: String,
    pub route_name: String,
    pub mission_kind: MissionKind,
    pub carriage: Carriage,
    pub guards: Vec<Guard>,
    pub enemies: Vec<Enemy>,
    pub hazards: Vec<Hazard>,
    pub shots: Vec<Shot>,
    /// Floating combat numbers (juice); short-lived, purely visual.
    pub float_texts: FloatingTextLayer,
    /// Burst particles (juice); short-lived, purely visual.
    pub particles: ParticleSystem,
    pub drag: DragState,
    pub alert: Alert,
    pub progress: f32,
    pub distance: f32,
    pub difficulty: f32,
    pub base_reward: i64,
    pub enemy_mix: Vec<String>,
    pub hazard_mix: Vec<String>,
    pub elapsed: f32,
    pub time_limit: Option<f32>,
    pub road_scroll: f32,
    pub terrain_scroll: f32,
    pub enemies_defeated: u32,
    pub damage_taken: f32,
    pub guard_damage_taken: f32,
    pub cargo_lost: f32,
    pub special_meter: f32,
    pub repair_used: bool,
    pub carriage_visual: CarriageVisual,
    /// Player throttle: >1 while boosting, <1 while braking, 1 at cruise.
    pub(super) throttle: f32,
    /// This frame's injected keyboard drive state.
    pub(super) drive: DriveKeys,
    /// Active chassis speed multiplier (Scout fast, Heavy slow).
    pub(super) chassis_speed_mult: f32,
    pub(super) wave: WavePhase,
    pub(super) wave_index: u32,
    pub(super) next_enemy_id: u32,
    pub(super) hazard_timer: f32,
    pub(super) rng: SeededRng,
    pub ranged_slots: usize,
    pub(super) armor_reduction: f32,
    pub(super) cargo_protection: f32,
    pub(super) wheel_bonus: f32,
    pub(super) repair_heal: f32,
    /// Contact damage per second dealt to enemies hugging the carriage (Spiked
    /// Hubs). Zero when not equipped.
    pub(super) hub_damage: f32,
    /// Radius within which the Warding Lantern slows enemies. Zero when not
    /// equipped.
    pub(super) ward_radius: f32,
    /// Multiplier on the lulls between enemy waves (>1 = gentler pacing). Driven
    /// by the Slower Waves accessibility assist.
    pub(super) wave_pace: f32,
    /// Monster-egg missions only: the shell has visibly cracked (telegraph).
    pub(super) egg_cracked: bool,
    /// Monster-egg missions only: the egg has hatched — the brood erupted and the
    /// stability meter is spent. Set once.
    pub(super) egg_hatched: bool,
    /// Princess-comfort missions only: the carriage's lateral offset from the
    /// road centre last frame, used to measure steering smoothness.
    pub(super) last_lateral: f32,
    /// Princess-comfort missions only: the smoothed "ride smoothness" multiplier
    /// (0..1). 1.0 = gliding clean; drops as you swerve. Drives comfort + score.
    pub(super) ride_smoothness: f32,
}

impl MissionRun {
    pub fn new(mission: &MissionDef, campaign: &CampaignState) -> Self {
        // Accessibility assists: sturdier carriage (+health) and gentler pacing.
        let assist_health = if campaign.sturdy_carriage { 1.25 } else { 1.0 };
        let wave_pace = if campaign.slower_waves { 1.5 } else { 1.0 };
        let max_health = (100.0 + campaign.carriage_level as f32 * 26.0)
            * campaign.chassis_health_mult
            * campaign.frame_health_mult
            * assist_health;
        let cargo_max = (100.0 + campaign.cargo_level as f32 * 6.0) * campaign.frame_cargo_mult;
        let route_choice = campaign.selected_route_choice(mission);
        let route_choice_id = route_choice
            .map(|choice| choice.id.clone())
            .unwrap_or_default();
        let route_seed = route_choice_id.bytes().fold(0_u64, |seed, byte| {
            seed.wrapping_mul(37).wrapping_add(byte as u64)
        });
        let seed = mission.order as u64 * 10_007
            + campaign
                .records
                .get(&mission.id)
                .map(|record| record.completions as u64)
                .unwrap_or(0)
            + route_seed;
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
        let difficulty = (route_choice
            .map(|choice| mission.difficulty + choice.difficulty_delta)
            .unwrap_or(mission.difficulty)
            .max(0.6)
            * campaign.difficulty_preset.difficulty_scale())
        .max(0.5);
        let base_reward = route_choice
            .map(|choice| mission.base_reward + choice.reward_delta)
            .unwrap_or(mission.base_reward)
            .max(0);
        let time_limit = mission.time_limit.map(|limit| {
            let base = route_choice
                .map(|choice| limit + choice.time_limit_delta)
                .unwrap_or(limit)
                .max(30.0);
            // Accessibility assist: extra seconds on the clock, orthogonal to
            // the difficulty preset.
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

        let armor_equipped = campaign.is_equipment_equipped(CarriageEquipment::IronPlating);
        let wheels_equipped = campaign.is_equipment_equipped(CarriageEquipment::ReinforcedWheels);
        let straps_equipped = campaign.is_equipment_equipped(CarriageEquipment::CargoStraps);
        let repair_equipped = campaign.is_equipment_equipped(CarriageEquipment::RepairKit);
        let hubs_equipped = campaign.is_equipment_equipped(CarriageEquipment::SpikedHubs);
        let lantern_equipped = campaign.is_equipment_equipped(CarriageEquipment::WardingLantern);

        Self {
            mission_id: mission.id.clone(),
            mission_name: mission.name.clone(),
            route_name: route_choice
                .map(|choice| choice.name.clone())
                .unwrap_or_else(|| mission.route.clone()),
            mission_kind,
            carriage: Carriage::new(max_health, cargo_max),
            guards,
            enemies: Vec::new(),
            hazards: Vec::new(),
            shots: Vec::new(),
            float_texts: {
                let mut layer = FloatingTextLayer::new();
                layer.default_lifetime = FLOAT_TEXT_LIFE;
                layer.default_rise_speed = 26.0;
                layer.shadow = false;
                layer
            },
            particles: ParticleSystem::new(),
            drag: DragState::None,
            alert: Alert::default(),
            progress: 0.0,
            distance,
            difficulty,
            base_reward,
            enemy_mix,
            hazard_mix,
            elapsed: 0.0,
            time_limit,
            road_scroll: 0.0,
            terrain_scroll: 0.0,
            enemies_defeated: 0,
            damage_taken: 0.0,
            guard_damage_taken: 0.0,
            cargo_lost: 0.0,
            special_meter: match mission_kind {
                MissionKind::CargoTransfer
                | MissionKind::PrisonerEscort
                | MissionKind::TimeDelivery => 0.0,
                _ => 100.0,
            },
            repair_used: false,
            carriage_visual: CarriageVisual::from_campaign(campaign),
            throttle: 1.0,
            drive: DriveKeys::default(),
            chassis_speed_mult: campaign.chassis_speed_mult * campaign.frame_speed_mult,
            // Siege runs open with a longer calm before the first mega-wave.
            wave: WavePhase::Lull(
                2.2 * wave_pace
                    * if mission_kind == MissionKind::SiegeSupplyRun {
                        2.0
                    } else {
                        1.0
                    },
            ),
            wave_index: 0,
            next_enemy_id: 10,
            hazard_timer: 1.6,
            rng: SeededRng::new(seed),
            ranged_slots,
            armor_reduction: if armor_equipped {
                campaign.carriage_level as f32 * 1.8
            } else {
                campaign.carriage_level as f32 * 0.45
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
            wave_pace,
            egg_cracked: false,
            egg_hatched: false,
            last_lateral: 0.0,
            ride_smoothness: 1.0,
        }
    }

    /// Applies expedition modifiers to a freshly-built leg: harder enemies and
    /// a carriage that starts at its carried-over health rather than full.
    pub fn scale_for_journey(&mut self, difficulty_scale: f32, health_ratio: f32) {
        self.difficulty *= difficulty_scale;
        self.carriage.health = (self.carriage.max_health * health_ratio).max(1.0);
    }

    /// Folds a bespoke expedition-leg modifier into this run: extra enemies and
    /// hazards in the spawn pools, scaled difficulty and banked reward. Applied
    /// once when composing a procedural leg (see `GameSession::begin_journey_leg`).
    pub fn apply_leg_modifier(&mut self, modifier: &crate::data::LegModifierDef) {
        self.enemy_mix.extend(modifier.enemy_add.iter().cloned());
        self.hazard_mix.extend(modifier.hazard_add.iter().cloned());
        self.difficulty = (self.difficulty * modifier.difficulty_mult).max(0.5);
        self.base_reward = ((self.base_reward as f32) * modifier.reward_mult).round() as i64;
    }

    /// Folds a collected expedition relic's modifiers into this run. Applied per
    /// leg on top of chassis/equipment stats (see `GameSession::begin_journey_leg`).
    pub fn apply_relic(&mut self, relic: &crate::data::RelicDef) {
        self.chassis_speed_mult *= relic.speed_mult;
        self.armor_reduction = (self.armor_reduction + relic.armor_add).clamp(0.0, 0.9);
        self.wheel_bonus += relic.wheel_bonus_add;
        self.hub_damage += relic.hub_damage_add;
    }

    pub fn progress_ratio(&self) -> f32 {
        (self.progress / self.distance.max(1.0)).clamp(0.0, 1.0)
    }

    pub fn speed_factor(&self) -> f32 {
        if self.carriage.slow_timer > 0.0 {
            (0.60 + self.wheel_bonus * 0.06).min(0.9)
        } else {
            1.0
        }
    }

    pub fn scroll_speed(&self) -> f32 {
        (Self::BASE_SCROLL_SPEED + self.wheel_bonus * 9.0)
            * self.speed_factor()
            * self.throttle
            * self.chassis_speed_mult
    }

    /// Cruising scroll speed with no wheel upgrades and no slowdown, in px/sec.
    pub const BASE_SCROLL_SPEED: f32 = 128.0;

    /// Stylized speed readout for the HUD; base cruising speed reads ~18.
    pub fn speed_readout(&self) -> f32 {
        self.scroll_speed() / Self::BASE_SCROLL_SPEED * 18.0
    }

    /// Fraction of the speed gauge to fill (full wheel upgrades approach 1.0).
    pub fn speed_ratio(&self) -> f32 {
        (self.scroll_speed() / (Self::BASE_SCROLL_SPEED * 1.4)).clamp(0.0, 1.0)
    }

    pub fn is_slowed(&self) -> bool {
        self.carriage.slow_timer > 0.0
    }

    pub fn is_boosted(&self) -> bool {
        !self.is_slowed() && self.throttle > 1.02
    }

    /// True while the player is actively holding the brake (not mud-slowed).
    pub fn is_braking(&self) -> bool {
        !self.is_slowed() && self.throttle < 0.98
    }

    /// The wave number being telegraphed, if a warning is currently showing.
    pub fn wave_telegraph(&self) -> Option<u32> {
        matches!(self.wave, WavePhase::Telegraph(_)).then_some(self.wave_index)
    }

    /// The live ride-smoothness multiplier (1.0–2.0) for princess-comfort runs,
    /// where scoring rewards driving clean. `None` on other mission types.
    pub fn ride_smoothness_multiplier(&self) -> Option<f32> {
        (self.mission_kind == MissionKind::PrincessEscort)
            .then_some(1.0 + self.ride_smoothness.clamp(0.0, 1.0))
    }

    pub fn special_ratio(&self) -> Option<f32> {
        match self.mission_kind {
            MissionKind::CargoTransfer => None,
            MissionKind::PrisonerEscort => Some((1.0 - self.special_meter / 100.0).clamp(0.0, 1.0)),
            MissionKind::TimeDelivery => self
                .time_limit
                .map(|limit| ((limit - self.elapsed) / limit.max(1.0)).clamp(0.0, 1.0)),
            _ => Some((self.special_meter / 100.0).clamp(0.0, 1.0)),
        }
    }

    /// Spend a Reinforced Kit: a one-route boost to maximum health, applied at
    /// full so the carriage sets out sturdier.
    pub fn apply_reinforced_kit(&mut self) {
        self.carriage.max_health += REINFORCED_KIT_HEALTH;
        self.carriage.health = self.carriage.max_health;
    }

    pub fn repair_available(&self) -> bool {
        self.repair_heal > 0.0
            && !self.repair_used
            && self.carriage.health < self.carriage.max_health
    }

    pub fn use_emergency_repair(&mut self) -> bool {
        if !self.repair_available() {
            return false;
        }

        self.repair_used = true;
        self.carriage.health =
            (self.carriage.health + self.repair_heal).min(self.carriage.max_health);
        self.carriage.hit_flash = Timer::new(0.28);
        self.alert.set("Emergency repair");
        true
    }

    pub fn carriage_slot_pos(&self, slot: usize) -> Vec2 {
        carriage_slot_pos(self.carriage.pos.x, slot, self.ranged_slots)
    }
}

pub(super) fn carriage_slot_pos(carriage_x: f32, slot: usize, total_slots: usize) -> Vec2 {
    let spacing = 30.0;
    let offset = slot as f32 * spacing - (total_slots.saturating_sub(1) as f32 * spacing * 0.5);
    vec2(carriage_x + offset, CARRIAGE_Y - 26.0)
}
