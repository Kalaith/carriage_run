//! Active route simulation for Carriage Run missions.

mod combat;
mod damage;
mod effects;
mod flow;
mod pressure;
mod scoring;
mod setup;

use super::entities::*;
use super::{BossState, CampaignState, CarriageVisual};
use crate::data::MissionDef;
use macroquad::prelude::*;
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
    pub reward_breakdown: RewardBreakdown,
    /// Gold lost to repairs and spoiled cargo when a run fails (0 on success).
    pub gold_penalty: i64,
    pub elapsed: f32,
    pub time_limit: Option<f32>,
    pub carriage_health_ratio: f32,
    pub cargo_ratio: f32,
    pub special_label: Option<String>,
    pub special_ratio: Option<f32>,
    pub enemies_defeated: u32,
    pub enemies_encountered: u32,
    pub hazards_encountered: u32,
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
    pub biome: String,
    pub hazard_palette: Vec<String>,
    pub mission_kind: MissionKind,
    pub carriage: Carriage,
    pub guards: Vec<Guard>,
    pub enemies: Vec<Enemy>,
    pub hazards: Vec<Hazard>,
    /// Optional finale encounter. The same state machine serves campaign and
    /// expedition finales.
    pub boss: Option<BossState>,
    pub shots: Vec<Shot>,
    /// Short-lived combat feedback; kept separate from authoritative route
    /// state so visual effects do not become additional simulation concerns.
    pub effects: effects::MissionEffects,
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
    pub enemies_encountered: u32,
    pub hazards_encountered: u32,
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
    /// Prisoner-escort breakout state: security attempts are telegraphed and
    /// can be interrupted by braking or a nearby guard.
    pub(super) breakout_timer: f32,
    pub(super) breakout_progress: f32,
    pub(super) breakout_attempts: u32,
    /// Princess-comfort missions only: the carriage's lateral offset from the
    /// road centre last frame, used to measure steering smoothness.
    pub(super) last_lateral: f32,
    /// Princess-comfort missions only: the smoothed "ride smoothness" multiplier
    /// (0..1). 1.0 = gliding clean; drops as you swerve. Drives comfort + score.
    pub(super) ride_smoothness: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RewardBreakdown {
    pub contract: i64,
    pub stars: i64,
    pub cargo: i64,
    pub special: i64,
    pub threats: i64,
    pub bonus_objective: i64,
}

#[derive(Debug, Clone, Copy)]
enum MissionRunContext {
    Campaign,
    Expedition { seed: u64 },
}

impl RewardBreakdown {
    pub fn total(self) -> i64 {
        self.contract + self.stars + self.cargo + self.special + self.threats + self.bonus_objective
    }
}

impl MissionRun {
    pub fn new(mission: &MissionDef, campaign: &CampaignState) -> Self {
        Self::new_with_context(mission, campaign, MissionRunContext::Campaign)
    }

    pub(crate) fn new_for_expedition(
        mission: &MissionDef,
        campaign: &CampaignState,
        seed: u64,
    ) -> Self {
        Self::new_with_context(mission, campaign, MissionRunContext::Expedition { seed })
    }

    fn new_with_context(
        mission: &MissionDef,
        campaign: &CampaignState,
        context: MissionRunContext,
    ) -> Self {
        let setup = setup::MissionSetup::resolve(mission, campaign, context);
        let mission_kind = setup.mission_kind;

        Self {
            mission_id: mission.id.clone(),
            mission_name: mission.name.clone(),
            route_name: setup.route_name,
            biome: mission.authored_biome().to_owned(),
            hazard_palette: mission.palette().to_owned(),
            mission_kind,
            carriage: Carriage::new(setup.max_health, setup.cargo_max),
            guards: setup.guards,
            enemies: Vec::new(),
            hazards: Vec::new(),
            boss: mission.boss_id.as_deref().map(BossState::new),
            shots: Vec::new(),
            effects: effects::MissionEffects::new(FLOAT_TEXT_LIFE),
            drag: DragState::None,
            alert: Alert::default(),
            progress: 0.0,
            distance: setup.distance,
            difficulty: setup.difficulty,
            base_reward: setup.base_reward,
            enemy_mix: setup.enemy_mix,
            hazard_mix: setup.hazard_mix,
            elapsed: 0.0,
            time_limit: setup.time_limit,
            road_scroll: 0.0,
            terrain_scroll: 0.0,
            enemies_defeated: 0,
            enemies_encountered: 0,
            hazards_encountered: 0,
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
            chassis_speed_mult: setup.chassis_speed_mult,
            // Siege runs open with a longer calm before the first mega-wave.
            wave: WavePhase::Lull(
                2.2 * setup.wave_pace
                    * if mission_kind == MissionKind::SiegeSupplyRun {
                        2.0
                    } else {
                        1.0
                    },
            ),
            wave_index: 0,
            next_enemy_id: 10,
            hazard_timer: 1.6,
            rng: SeededRng::new(setup.seed),
            ranged_slots: setup.ranged_slots,
            armor_reduction: setup.armor_reduction,
            cargo_protection: setup.cargo_protection,
            wheel_bonus: setup.wheel_bonus,
            repair_heal: setup.repair_heal,
            hub_damage: setup.hub_damage,
            ward_radius: setup.ward_radius,
            wave_pace: setup.wave_pace,
            egg_cracked: false,
            egg_hatched: false,
            breakout_timer: 6.0,
            breakout_progress: 0.0,
            breakout_attempts: 0,
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
        self.armor_reduction = (self.armor_reduction + relic.flat_armor_add).max(0.0);
        self.wheel_bonus += relic.wheel_bonus_add;
        self.hub_damage += relic.hub_damage_add;
    }

    pub fn progress_ratio(&self) -> f32 {
        (self.progress / self.distance.max(1.0)).clamp(0.0, 1.0)
    }

    pub fn speed_factor(&self) -> f32 {
        if self.carriage.night_timer > 0.0 {
            (0.78 + self.wheel_bonus * 0.04).min(0.92)
        } else if self.carriage.slow_timer > 0.0 {
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

    pub fn is_in_night_stretch(&self) -> bool {
        self.carriage.night_timer > 0.0
    }

    pub fn boss_status(&self) -> Option<(&str, &str, f32)> {
        self.boss.as_ref().map(|boss| {
            (
                boss.definition.name,
                boss.phase.label(),
                boss.health_ratio(),
            )
        })
    }

    pub fn breakout_status(&self) -> Option<(f32, bool)> {
        (self.mission_kind == MissionKind::PrisonerEscort).then_some((
            self.breakout_progress.clamp(0.0, 1.0),
            self.breakout_progress > 0.0,
        ))
    }

    pub fn screen_shake_offset(&self) -> Vec2 {
        self.effects.screen_shake.offset()
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
