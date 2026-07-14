//! Persistent campaign state and the shared vocabulary of screens and presets.

mod campaign;
mod chassis;
mod entities;
mod equipment;
mod journey;
mod mission;
mod save;
mod session;
mod validation;

pub use entities::*;
pub use equipment::*;
pub use journey::{ExpeditionRecords, ExpeditionRunSummary, Journey, LegReward};
pub use mission::{MissionInput, MissionReport, MissionRun};
pub use save::{migrate_save_value, SaveData};
pub use session::GameSession;
pub use validation::{validate_mission_content, validate_mission_reachability};

use crate::data::{GameConfig, UpgradeDef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    MissionMap,
    Loadout,
    Shop,
    Carriages,
    Guards,
    Upgrades,
    Settings,
    Playing,
    Paused,
    Results,
    Journey,
    Outfitter,
    Records,
    Codex,
}

/// A destructive action awaiting explicit player confirmation. Session-only
/// (never serialized) — it gates footguns like overwriting an existing save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmPrompt {
    /// Starting a new campaign would overwrite the current autosave.
    NewCampaign,
}

/// Gold cost of one Reinforced Kit consumable. A repeatable sink for gold that
/// is otherwise worthless once upgrades are maxed.
pub const REINFORCED_KIT_COST: i64 = 45;

/// Which section of the Field Guide is showing (session-only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodexTab {
    #[default]
    Threats,
    Guards,
    Hazards,
}

/// Player-chosen challenge level, applied as a multiplier on each mission's
/// difficulty scalar (which drives spawn rate, burst size, and enemy stats).
/// Widens the audience and doubles as an accessibility assist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DifficultyPreset {
    Relaxed,
    #[default]
    Standard,
    Hard,
}

impl DifficultyPreset {
    pub fn all() -> [Self; 3] {
        [Self::Relaxed, Self::Standard, Self::Hard]
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Relaxed => "relaxed",
            Self::Standard => "standard",
            Self::Hard => "hard",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Relaxed => "Relaxed",
            Self::Standard => "Standard",
            Self::Hard => "Hard",
        }
    }

    pub fn from_id(id: &str) -> Self {
        Self::all()
            .into_iter()
            .find(|preset| preset.id() == id)
            .unwrap_or_default()
    }

    /// Multiplier applied to mission difficulty. Fewer/weaker foes when relaxed,
    /// more/stronger when hard.
    pub fn difficulty_scale(self) -> f32 {
        match self {
            Self::Relaxed => 0.8,
            Self::Standard => 1.0,
            Self::Hard => 1.25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRecord {
    pub best_stars: u8,
    pub best_score: i64,
    pub best_reward: i64,
    pub completions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignState {
    pub gold: i64,
    pub carriage_level: u32,
    pub guard_level: u32,
    pub archer_level: u32,
    pub wheel_level: u32,
    pub cargo_level: u32,
    #[serde(default)]
    pub repair_level: u32,
    #[serde(default)]
    pub hubs_level: u32,
    #[serde(default)]
    pub lantern_level: u32,
    #[serde(default = "default_guard_id")]
    pub selected_guard_id: String,
    #[serde(default = "default_selected_guard_ids")]
    pub selected_guard_ids: Vec<String>,
    #[serde(default = "default_selected_ranged_ids")]
    pub selected_ranged_ids: Vec<String>,
    #[serde(default = "default_hired_guard_ids")]
    pub hired_guard_ids: Vec<String>,
    #[serde(default = "default_selected_equipment_ids")]
    pub selected_equipment_ids: Vec<String>,
    #[serde(default)]
    pub chassis_id: String,
    #[serde(default)]
    pub owned_chassis_ids: Vec<String>,
    /// Cached from the active chassis def (see `refresh_chassis_stats`). Zero
    /// means "not yet resolved" and falls back to the legacy carriage-level
    /// slot formula.
    #[serde(default)]
    pub chassis_slots: usize,
    #[serde(default = "default_mult")]
    pub chassis_speed_mult: f32,
    #[serde(default = "default_mult")]
    pub chassis_health_mult: f32,
    #[serde(default)]
    pub guard_stars: HashMap<String, u8>,
    #[serde(default)]
    pub guard_recovery: HashMap<String, u32>,
    #[serde(default = "default_true")]
    pub route_motion_enabled: bool,
    #[serde(default = "default_true")]
    pub alerts_enabled: bool,
    #[serde(default = "default_true")]
    pub auto_save_enabled: bool,
    #[serde(default)]
    pub difficulty_preset: DifficultyPreset,
    /// Accessibility assist: grant extra time on timed missions. Orthogonal to
    /// `difficulty_preset` (which scales enemies, not the clock).
    #[serde(default)]
    pub generous_timers: bool,
    /// Reinforced Kit consumables in stock; one is spent (for +health) at the
    /// start of each campaign route.
    #[serde(default)]
    pub reinforced_kits: u32,
    /// Accessibility assist: gentler wave pacing (longer lulls between attacks).
    /// Orthogonal to the difficulty preset.
    #[serde(default)]
    pub slower_waves: bool,
    /// Accessibility assist: the carriage sets out with bonus max health.
    #[serde(default)]
    pub sturdy_carriage: bool,
    /// Persistent expedition currency (meta-progression). Earned per leg cleared
    /// across all expeditions; spent at the Outfitter on permanent unlocks.
    #[serde(default)]
    pub expedition_tokens: i64,
    /// Relic ids permanently unlocked as expedition starting boons; every future
    /// expedition begins with these.
    #[serde(default)]
    pub expedition_unlocks: Vec<String>,
    /// Persistent expedition stats + recent-run history (Records screen).
    #[serde(default)]
    pub expedition_records: ExpeditionRecords,
    /// Chosen expedition entry-stake tier id (Outfitter). Persisted so the last
    /// choice sticks; defaults to the no-stake tier.
    #[serde(default = "default_stake_id")]
    pub selected_stake_id: String,
    /// Chosen mutually-exclusive carriage frame tuning id (Carriages screen).
    /// Exactly one is active; defaults to the balanced Standard Frame.
    #[serde(default = "default_frame_id")]
    pub carriage_frame_id: String,
    /// Resolved frame-tuning multipliers (refreshed from `carriage_frame_id`).
    #[serde(default = "one_f32")]
    pub frame_speed_mult: f32,
    #[serde(default = "one_f32")]
    pub frame_health_mult: f32,
    #[serde(default = "one_f32")]
    pub frame_cargo_mult: f32,
    pub selected_mission_id: String,
    #[serde(default)]
    pub selected_route_choices: HashMap<String, String>,
    pub records: HashMap<String, MissionRecord>,
}

impl CampaignState {
    pub fn new(config: &GameConfig, first_mission_id: Option<&str>) -> Self {
        Self {
            gold: config.starting_gold,
            carriage_level: 1,
            guard_level: 1,
            archer_level: 1,
            wheel_level: 0,
            cargo_level: 0,
            repair_level: 0,
            hubs_level: 0,
            lantern_level: 0,
            selected_guard_id: default_guard_id(),
            selected_guard_ids: default_selected_guard_ids(),
            selected_ranged_ids: default_selected_ranged_ids(),
            hired_guard_ids: default_hired_guard_ids(),
            selected_equipment_ids: default_selected_equipment_ids(),
            chassis_id: String::new(),
            owned_chassis_ids: Vec::new(),
            chassis_slots: 0,
            chassis_speed_mult: 1.0,
            chassis_health_mult: 1.0,
            guard_stars: HashMap::new(),
            guard_recovery: HashMap::new(),
            route_motion_enabled: true,
            alerts_enabled: true,
            auto_save_enabled: true,
            difficulty_preset: DifficultyPreset::Standard,
            generous_timers: false,
            reinforced_kits: 0,
            slower_waves: false,
            sturdy_carriage: false,
            expedition_tokens: 0,
            expedition_unlocks: Vec::new(),
            expedition_records: ExpeditionRecords::default(),
            selected_stake_id: default_stake_id(),
            carriage_frame_id: default_frame_id(),
            frame_speed_mult: 1.0,
            frame_health_mult: 1.0,
            frame_cargo_mult: 1.0,
            selected_mission_id: first_mission_id.unwrap_or("muddy_road").to_owned(),
            selected_route_choices: HashMap::new(),
            records: HashMap::new(),
        }
    }

    pub fn upgrade_level(&self, id: &str) -> u32 {
        match id {
            "carriage_armor" => self.carriage_level,
            "guard_training" => self.guard_level,
            "mounted_archer" => self.archer_level,
            "reinforced_wheels" => self.wheel_level,
            "cargo_straps" => self.cargo_level,
            "repair_kit" => self.repair_level,
            "spiked_hubs" => self.hubs_level,
            "warding_lantern" => self.lantern_level,
            _ => 0,
        }
    }

    pub fn upgrade_cost(&self, upgrade: &UpgradeDef) -> Option<i64> {
        let level = self.upgrade_level(&upgrade.id);
        (level < upgrade.max_level).then_some(upgrade.base_cost * (level as i64 + 1))
    }

    pub fn selected_guard_kind(&self) -> GuardKind {
        GuardKind::from_id(&self.selected_guard_id)
    }

    pub fn selected_melee_kinds(&self) -> Vec<GuardKind> {
        self.selected_guard_ids
            .iter()
            .map(|id| GuardKind::from_id(id))
            .filter(|kind| kind.is_melee() && self.is_guard_hired(*kind))
            .take(self.guard_slot_count())
            .collect()
    }

    pub fn selected_ranged_kinds(&self) -> Vec<GuardKind> {
        self.selected_ranged_ids
            .iter()
            .map(|id| GuardKind::from_id(id))
            .filter(|kind| kind.is_ranged() && self.is_guard_hired(*kind))
            .take(self.ranged_slot_count())
            .collect()
    }

    pub fn guard_slot_count(&self) -> usize {
        self.chassis_slot_count()
    }

    pub fn ranged_slot_count(&self) -> usize {
        if self.archer_level >= 3 {
            2
        } else {
            1
        }
    }

    pub fn is_guard_unlocked(&self, kind: GuardKind) -> bool {
        self.carriage_level >= kind.unlock_level()
    }

    pub fn is_guard_hired(&self, kind: GuardKind) -> bool {
        self.hired_guard_ids
            .iter()
            .any(|id| id.as_str() == kind.id())
    }

    pub fn guard_hire_cost(&self, kind: GuardKind) -> i64 {
        kind.hire_cost()
    }

    pub fn can_hire_guard(&self, kind: GuardKind) -> bool {
        self.is_guard_unlocked(kind)
            && !self.is_guard_hired(kind)
            && self.gold >= self.guard_hire_cost(kind)
    }

    pub fn guard_star_level(&self, kind: GuardKind) -> u8 {
        self.guard_stars
            .get(kind.id())
            .copied()
            .unwrap_or(1)
            .clamp(1, 3)
    }

    pub fn guard_star_upgrade_cost(&self, kind: GuardKind) -> Option<i64> {
        kind.star_upgrade_cost(self.guard_star_level(kind))
    }

    pub fn guard_recovery_missions(&self, kind: GuardKind) -> u32 {
        self.guard_recovery.get(kind.id()).copied().unwrap_or(0)
    }

    /// Gold cost to instantly treat an injured guard at the infirmary. `None`
    /// when the guard is not currently recovering.
    pub fn guard_treat_cost(&self, kind: GuardKind) -> Option<i64> {
        let recovery = self.guard_recovery_missions(kind);
        (recovery > 0).then(|| 30 + kind.hire_cost() / 5 + recovery as i64 * 15)
    }

    pub fn is_guard_available(&self, kind: GuardKind) -> bool {
        self.is_guard_hired(kind) && self.guard_recovery_missions(kind) == 0
    }

    pub fn normalize(&mut self, first_mission_id: Option<&str>) {
        if self.selected_mission_id.is_empty() {
            self.selected_mission_id = first_mission_id.unwrap_or("muddy_road").to_owned();
        }
        self.selected_route_choices
            .retain(|mission_id, route_id| !mission_id.is_empty() && !route_id.is_empty());

        if self.hired_guard_ids.is_empty() {
            self.hired_guard_ids = default_hired_guard_ids();
        }
        if !self.hired_guard_ids.iter().any(|id| id == "archer") {
            self.hired_guard_ids.push("archer".to_owned());
        }

        let mut unique = Vec::new();
        for kind in GuardKind::all() {
            if self
                .hired_guard_ids
                .iter()
                .any(|id| id.as_str() == kind.id())
                && !unique.iter().any(|id: &String| id == kind.id())
            {
                unique.push(kind.id().to_owned());
            }
        }
        if !unique.iter().any(|id| id == "swordsman") {
            unique.insert(0, default_guard_id());
        }
        if !unique.iter().any(|id| id == "archer") {
            unique.push("archer".to_owned());
        }
        self.hired_guard_ids = unique;

        let selected = GuardKind::from_id(&self.selected_guard_id);
        if !self.is_guard_hired(selected) {
            self.selected_guard_id = default_guard_id();
        }

        self.selected_guard_ids = normalize_selection(
            &self.selected_guard_ids,
            std::slice::from_ref(&self.selected_guard_id),
            self.guard_slot_count(),
            false,
            &self.hired_guard_ids,
        );
        if self.selected_guard_ids.is_empty() {
            self.selected_guard_ids = default_selected_guard_ids();
        }
        self.selected_guard_id = self
            .selected_guard_ids
            .first()
            .cloned()
            .unwrap_or_else(default_guard_id);

        self.selected_ranged_ids = normalize_selection(
            &self.selected_ranged_ids,
            &default_selected_ranged_ids(),
            self.ranged_slot_count(),
            true,
            &self.hired_guard_ids,
        );
        if self.selected_ranged_ids.is_empty() {
            self.selected_ranged_ids = default_selected_ranged_ids();
        }

        let hired_guard_ids = self.hired_guard_ids.clone();
        self.guard_stars.retain(|id, stars| {
            let kind = GuardKind::from_id(id);
            *id == kind.id()
                && hired_guard_ids.iter().any(|hired_id| hired_id == kind.id())
                && *stars >= 1
        });
        for stars in self.guard_stars.values_mut() {
            *stars = (*stars).clamp(1, 3);
        }
        self.guard_recovery
            .retain(|id, turns| *turns > 0 && GuardKind::from_id(id).id() == id.as_str());
        self.normalize_equipment();
    }

    fn add_upgrade_level(&mut self, id: &str) {
        match id {
            "carriage_armor" => self.carriage_level += 1,
            "guard_training" => self.guard_level += 1,
            "mounted_archer" => self.archer_level += 1,
            "reinforced_wheels" => self.wheel_level += 1,
            "cargo_straps" => self.cargo_level += 1,
            "repair_kit" => self.repair_level += 1,
            "spiked_hubs" => self.hubs_level += 1,
            "warding_lantern" => self.lantern_level += 1,
            _ => {}
        }
    }
}

fn default_guard_id() -> String {
    "swordsman".to_owned()
}

fn default_stake_id() -> String {
    "none".to_owned()
}

fn default_frame_id() -> String {
    "standard".to_owned()
}

fn one_f32() -> f32 {
    1.0
}

fn default_selected_guard_ids() -> Vec<String> {
    vec![default_guard_id()]
}

fn default_selected_ranged_ids() -> Vec<String> {
    vec!["archer".to_owned()]
}

fn default_hired_guard_ids() -> Vec<String> {
    vec![default_guard_id(), "archer".to_owned()]
}

fn default_true() -> bool {
    true
}

fn default_mult() -> f32 {
    1.0
}

fn normalize_selection(
    current: &[String],
    fallback: &[String],
    limit: usize,
    ranged: bool,
    hired: &[String],
) -> Vec<String> {
    let mut selected = Vec::new();
    for id in current.iter().chain(fallback.iter()) {
        let kind = GuardKind::from_id(id);
        if kind.is_ranged() != ranged || !hired.iter().any(|hired_id| hired_id == kind.id()) {
            continue;
        }
        if selected
            .iter()
            .any(|selected_id: &String| selected_id == kind.id())
        {
            continue;
        }
        selected.push(kind.id().to_owned());
        if selected.len() >= limit {
            break;
        }
    }
    selected
}

#[cfg(test)]
mod tests;
