//! Campaign state, save data, and screen-level mission orchestration.

mod campaign;
mod chassis;
mod entities;
mod equipment;
mod journey;
mod mission;

pub use entities::*;
pub use equipment::*;
pub use journey::{ExpeditionRecords, ExpeditionRunSummary, Journey, LegReward};
pub use mission::{MissionInput, MissionReport, MissionRun};

use crate::data::{GameConfig, GameData, MissionDef, UpgradeDef};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

/// Fail fast on content typos: every enemy/hazard id referenced by mission
/// data must resolve to a known kind. Unknown ids otherwise degrade silently
/// to Wolf/Mud at spawn time (`mission/flow.rs`), shipping the wrong encounter
/// with no error. Guarded by a unit test (CI) and a debug-build assert.
pub fn validate_mission_content(missions: &[&MissionDef]) -> Result<(), String> {
    let mut unknown = Vec::new();
    let check_enemy = |unknown: &mut Vec<String>, where_: &str, id: &str| {
        if EnemyKind::from_id(id).is_none() {
            unknown.push(format!("{where_}: enemy '{id}'"));
        }
    };
    let check_hazard = |unknown: &mut Vec<String>, where_: &str, id: &str| {
        if HazardKind::from_id(id).is_none() {
            unknown.push(format!("{where_}: hazard '{id}'"));
        }
    };

    for mission in missions {
        for id in &mission.enemy_mix {
            check_enemy(&mut unknown, &mission.id, id);
        }
        for id in &mission.hazard_mix {
            check_hazard(&mut unknown, &mission.id, id);
        }
        for choice in &mission.route_choices {
            let where_ = format!("{}/{}", mission.id, choice.id);
            for id in &choice.enemy_add {
                check_enemy(&mut unknown, &where_, id);
            }
            for id in &choice.hazard_add {
                check_hazard(&mut unknown, &where_, id);
            }
        }
    }

    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!("unknown content ids -> {}", unknown.join(", ")))
    }
}

/// Verify the mission unlock graph is sound: every prerequisite/branch id
/// refers to a real mission, and every mission can eventually be unlocked from
/// a fresh campaign. A typo'd prerequisite otherwise silently strands a mission
/// as permanently locked. (Carriage-level gates are always satisfiable via
/// upgrades, so only completion prerequisites constrain reachability.)
pub fn validate_mission_reachability(missions: &[&MissionDef]) -> Result<(), String> {
    use std::collections::HashSet;

    let ids: HashSet<&str> = missions.iter().map(|mission| mission.id.as_str()).collect();
    let mut errors = Vec::new();

    for mission in missions {
        for id in mission
            .prerequisite_missions
            .iter()
            .chain(mission.unlock_any_missions.iter())
        {
            if !ids.contains(id.as_str()) {
                errors.push(format!(
                    "{}: references unknown mission '{}'",
                    mission.id, id
                ));
            }
        }
    }

    // Fixpoint: a mission unlocks once all its prerequisites are reachable and
    // (if it has a branch requirement) at least one branch is reachable.
    let mut reachable: HashSet<&str> = HashSet::new();
    loop {
        let mut changed = false;
        for mission in missions {
            if reachable.contains(mission.id.as_str()) {
                continue;
            }
            let prereqs_ok = mission
                .prerequisite_missions
                .iter()
                .all(|id| reachable.contains(id.as_str()));
            let branch_ok = mission.unlock_any_missions.is_empty()
                || mission
                    .unlock_any_missions
                    .iter()
                    .any(|id| reachable.contains(id.as_str()));
            if prereqs_ok && branch_ok {
                reachable.insert(mission.id.as_str());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for mission in missions {
        if !reachable.contains(mission.id.as_str()) {
            errors.push(format!(
                "{}: unreachable (its prerequisites can never all be met)",
                mission.id
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("mission graph invalid -> {}", errors.join("; ")))
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub campaign: CampaignState,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub campaign: CampaignState,
    pub screen: Screen,
    pub mission: Option<MissionRun>,
    pub result: Option<MissionReport>,
    /// Active roguelite expedition, if one is in progress (session-only).
    pub journey: Option<Journey>,
    /// A destructive action awaiting player confirmation (session-only).
    pub pending_confirm: Option<ConfirmPrompt>,
    /// Active Field Guide tab (session-only).
    pub codex_tab: CodexTab,
}

impl GameSession {
    pub fn new(config: &GameConfig, first_mission_id: Option<&str>) -> Self {
        Self {
            campaign: CampaignState::new(config, first_mission_id),
            screen: Screen::Title,
            mission: None,
            result: None,
            journey: None,
            pending_confirm: None,
            codex_tab: CodexTab::Threats,
        }
    }

    pub fn from_save(save: SaveData, first_mission_id: Option<&str>) -> Self {
        let mut campaign = save.campaign;
        campaign.normalize(first_mission_id);

        Self {
            campaign,
            screen: Screen::MissionMap,
            mission: None,
            result: None,
            journey: None,
            pending_confirm: None,
            codex_tab: CodexTab::Threats,
        }
    }

    /// Begin a New Campaign request. When an existing save would be overwritten
    /// we stage a confirmation prompt and return `false`; with nothing to lose
    /// the caller may proceed immediately (returns `true`).
    pub fn request_new_campaign(&mut self, save_exists: bool) -> bool {
        if save_exists {
            self.pending_confirm = Some(ConfirmPrompt::NewCampaign);
            false
        } else {
            self.pending_confirm = None;
            true
        }
    }

    /// Dismiss any pending confirmation without acting on it.
    pub fn cancel_confirm(&mut self) {
        self.pending_confirm = None;
    }

    pub fn to_save(&self, version: &str) -> SaveData {
        SaveData {
            version: version.to_owned(),
            campaign: self.campaign.clone(),
        }
    }

    pub fn open_map(&mut self) {
        self.screen = Screen::MissionMap;
        self.mission = None;
    }

    pub fn open_loadout(&mut self) {
        self.screen = Screen::Loadout;
        self.mission = None;
    }

    pub fn open_upgrades(&mut self) {
        self.screen = Screen::Upgrades;
        self.mission = None;
    }

    pub fn open_shop(&mut self) {
        self.screen = Screen::Shop;
        self.mission = None;
    }

    pub fn open_carriages(&mut self) {
        self.screen = Screen::Carriages;
        self.mission = None;
    }

    pub fn open_guards(&mut self) {
        self.screen = Screen::Guards;
        self.mission = None;
    }

    pub fn open_settings(&mut self) {
        self.screen = Screen::Settings;
    }

    pub fn open_outfitter(&mut self) {
        self.screen = Screen::Outfitter;
    }

    pub fn open_records(&mut self) {
        self.screen = Screen::Records;
    }

    pub fn open_codex(&mut self) {
        self.screen = Screen::Codex;
        self.codex_tab = CodexTab::Threats;
    }

    pub fn set_codex_tab(&mut self, tab: CodexTab) {
        self.codex_tab = tab;
    }

    pub fn pause_play(&mut self) {
        if self.screen == Screen::Playing && self.mission.is_some() {
            self.screen = Screen::Paused;
        }
    }

    pub fn resume_play(&mut self) {
        if self.mission.is_some() {
            self.screen = Screen::Playing;
        }
    }

    pub fn return_title(&mut self) {
        self.screen = Screen::Title;
        self.mission = None;
    }

    pub fn select_mission(&mut self, id: &str) {
        self.campaign.selected_mission_id = id.to_owned();
    }

    pub fn select_route_choice(&mut self, data: &GameData, route_id: &str) -> bool {
        let Some(mission) = data.missions.get(&self.campaign.selected_mission_id) else {
            return false;
        };
        self.campaign.select_route_choice(mission, route_id)
    }

    pub fn select_guard(&mut self, id: &str) {
        let kind = GuardKind::from_id(id);
        if kind.is_melee()
            && self.campaign.is_guard_unlocked(kind)
            && self.campaign.is_guard_hired(kind)
        {
            self.campaign.selected_guard_id = kind.id().to_owned();
            self.assign_guard_slot(0, id);
        }
    }

    pub fn assign_guard_slot(&mut self, slot: usize, id: &str) {
        let kind = GuardKind::from_id(id);
        if kind.is_ranged()
            || slot >= self.campaign.guard_slot_count()
            || !self.campaign.is_guard_available(kind)
        {
            return;
        }
        self.campaign
            .selected_guard_ids
            .retain(|existing| existing != kind.id());
        if slot >= self.campaign.selected_guard_ids.len() {
            self.campaign
                .selected_guard_ids
                .resize(slot + 1, String::new());
        }
        self.campaign.selected_guard_ids[slot] = kind.id().to_owned();
        self.campaign
            .selected_guard_ids
            .retain(|existing| !existing.is_empty());
        self.campaign.selected_guard_id = self
            .campaign
            .selected_guard_ids
            .first()
            .cloned()
            .unwrap_or_else(default_guard_id);
    }

    pub fn clear_guard_slot(&mut self, slot: usize) {
        if slot < self.campaign.selected_guard_ids.len() {
            self.campaign.selected_guard_ids.remove(slot);
        }
        self.campaign.selected_guard_id = self
            .campaign
            .selected_guard_ids
            .first()
            .cloned()
            .unwrap_or_else(default_guard_id);
    }

    pub fn assign_ranged_slot(&mut self, slot: usize, id: &str) {
        let kind = GuardKind::from_id(id);
        if kind.is_melee()
            || slot >= self.campaign.ranged_slot_count()
            || !self.campaign.is_guard_available(kind)
        {
            return;
        }
        self.campaign
            .selected_ranged_ids
            .retain(|existing| existing != kind.id());
        if slot >= self.campaign.selected_ranged_ids.len() {
            self.campaign
                .selected_ranged_ids
                .resize(slot + 1, String::new());
        }
        self.campaign.selected_ranged_ids[slot] = kind.id().to_owned();
        self.campaign
            .selected_ranged_ids
            .retain(|existing| !existing.is_empty());
    }

    pub fn clear_ranged_slot(&mut self, slot: usize) {
        if slot < self.campaign.selected_ranged_ids.len() {
            self.campaign.selected_ranged_ids.remove(slot);
        }
    }

    pub fn hire_guard(&mut self, id: &str) -> bool {
        let kind = GuardKind::from_id(id);
        if !self.campaign.can_hire_guard(kind) {
            return false;
        }

        self.campaign.gold -= self.campaign.guard_hire_cost(kind);
        self.campaign.hired_guard_ids.push(kind.id().to_owned());
        if kind.is_ranged() {
            let slot = self.campaign.selected_ranged_ids.len();
            self.assign_ranged_slot(slot.min(self.campaign.ranged_slot_count() - 1), kind.id());
        } else {
            let slot = self.campaign.selected_guard_ids.len();
            self.assign_guard_slot(slot.min(self.campaign.guard_slot_count() - 1), kind.id());
        }
        true
    }

    pub fn upgrade_guard_star(&mut self, id: &str) -> bool {
        let kind = GuardKind::from_id(id);
        if !self.campaign.is_guard_hired(kind) {
            return false;
        }
        let Some(cost) = self.campaign.guard_star_upgrade_cost(kind) else {
            return false;
        };
        if self.campaign.gold < cost {
            return false;
        }

        self.campaign.gold -= cost;
        let next = self.campaign.guard_star_level(kind) + 1;
        self.campaign
            .guard_stars
            .insert(kind.id().to_owned(), next.min(3));
        true
    }

    /// Pay to clear an injured guard's recovery time immediately.
    pub fn treat_guard(&mut self, id: &str) -> bool {
        let kind = GuardKind::from_id(id);
        let Some(cost) = self.campaign.guard_treat_cost(kind) else {
            return false;
        };
        if self.campaign.gold < cost {
            return false;
        }

        self.campaign.gold -= cost;
        self.campaign.guard_recovery.remove(kind.id());
        true
    }

    pub fn toggle_setting(&mut self, id: &str) -> bool {
        match id {
            "route_motion" => {
                self.campaign.route_motion_enabled = !self.campaign.route_motion_enabled
            }
            "alerts" => self.campaign.alerts_enabled = !self.campaign.alerts_enabled,
            "auto_save" => self.campaign.auto_save_enabled = !self.campaign.auto_save_enabled,
            "generous_timers" => self.campaign.generous_timers = !self.campaign.generous_timers,
            "slower_waves" => self.campaign.slower_waves = !self.campaign.slower_waves,
            "sturdy_carriage" => self.campaign.sturdy_carriage = !self.campaign.sturdy_carriage,
            _ => return false,
        }
        true
    }

    /// Set the difficulty preset. Returns `true` when it actually changed.
    pub fn set_difficulty(&mut self, id: &str) -> bool {
        let preset = DifficultyPreset::from_id(id);
        if preset == self.campaign.difficulty_preset {
            return false;
        }
        self.campaign.difficulty_preset = preset;
        true
    }

    pub fn buy_upgrade(&mut self, upgrade: &UpgradeDef) -> bool {
        let Some(cost) = self.campaign.upgrade_cost(upgrade) else {
            return false;
        };
        if self.campaign.gold < cost {
            return false;
        }

        self.campaign.gold -= cost;
        self.campaign.add_upgrade_level(&upgrade.id);
        self.campaign.normalize_equipment();
        true
    }

    pub fn start_selected_mission(&mut self, data: &GameData) -> bool {
        let Some(mission) = data.missions.get(&self.campaign.selected_mission_id) else {
            return false;
        };
        if !self.campaign.is_mission_unlocked(mission) {
            return false;
        }

        let mut run = MissionRun::new(mission, &self.campaign);
        // Spend a Reinforced Kit if one is in stock, for a one-route boost.
        if self.campaign.reinforced_kits > 0 {
            self.campaign.reinforced_kits -= 1;
            run.apply_reinforced_kit();
        }
        self.mission = Some(run);
        self.result = None;
        self.screen = Screen::Playing;
        true
    }

    /// Purchase one Reinforced Kit if the player can afford it.
    pub fn buy_reinforced_kit(&mut self) -> bool {
        if self.campaign.gold < REINFORCED_KIT_COST {
            return false;
        }
        self.campaign.gold -= REINFORCED_KIT_COST;
        self.campaign.reinforced_kits += 1;
        true
    }

    pub fn retry_result_mission(&mut self, data: &GameData) -> bool {
        let Some(result) = &self.result else {
            return false;
        };
        self.campaign.selected_mission_id = result.mission_id.clone();
        self.start_selected_mission(data)
    }

    pub fn use_repair(&mut self) -> bool {
        self.mission
            .as_mut()
            .is_some_and(MissionRun::use_emergency_repair)
    }

    pub fn update_play(
        &mut self,
        data: &GameData,
        dt: f32,
        input: MissionInput,
    ) -> Option<MissionReport> {
        if self.screen != Screen::Playing {
            return None;
        }

        let mission_id = self.mission.as_ref()?.mission_id.clone();
        let mission = data.missions.get(&mission_id)?;
        let report = {
            let run = self.mission.as_mut()?;
            run.handle_input(input);
            run.update(mission, dt)
        };

        if let Some(report) = report {
            if self.journey.is_some() {
                self.resolve_journey_leg(&report, data);
            } else {
                self.apply_report(report.clone());
                self.result = Some(report.clone());
                self.mission = None;
                self.screen = Screen::Results;
            }
            Some(report)
        } else {
            None
        }
    }

    fn apply_report(&mut self, report: MissionReport) {
        self.advance_guard_recovery();
        self.campaign.gold = (self.campaign.gold + report.reward - report.gold_penalty).max(0);
        // Guards that fall are benched; a botched run leaves them worse off.
        let recovery = if report.success { 2 } else { 3 };
        for id in &report.injured_guard_ids {
            let kind = GuardKind::from_id(id);
            if self.campaign.is_guard_hired(kind) {
                self.campaign
                    .guard_recovery
                    .insert(kind.id().to_owned(), recovery);
            }
        }

        if report.success {
            let record = self
                .campaign
                .records
                .entry(report.mission_id.clone())
                .or_insert(MissionRecord {
                    best_stars: 0,
                    best_score: 0,
                    best_reward: 0,
                    completions: 0,
                });
            record.completions += 1;
            record.best_stars = record.best_stars.max(report.stars);
            record.best_score = record.best_score.max(report.score);
            record.best_reward = record.best_reward.max(report.reward);
        }
    }

    fn advance_guard_recovery(&mut self) {
        for turns in self.campaign.guard_recovery.values_mut() {
            *turns = turns.saturating_sub(1);
        }
        self.campaign.guard_recovery.retain(|_, turns| *turns > 0);
    }
}

#[derive(Debug, Deserialize)]
struct LegacyTemplateSave {
    points: Option<i64>,
}

pub fn migrate_save_value(
    detected_version: Option<String>,
    value: Value,
    config: &GameConfig,
    first_mission_id: Option<&str>,
) -> Result<SaveData, String> {
    let payload = value.get("data").cloned().unwrap_or(value);

    if let Ok(mut current) = serde_json::from_value::<SaveData>(payload.clone()) {
        current.version = config.version.clone();
        current.campaign.normalize(first_mission_id);
        return Ok(current);
    }

    if let Ok(mut campaign) = serde_json::from_value::<CampaignState>(payload.clone()) {
        campaign.normalize(first_mission_id);
        return Ok(SaveData {
            version: config.version.clone(),
            campaign,
        });
    }

    let legacy: LegacyTemplateSave = serde_json::from_value(payload)
        .map_err(|err| format!("Unsupported save format {:?}: {}", detected_version, err))?;
    let mut campaign = CampaignState::new(config, first_mission_id);
    if let Some(points) = legacy.points {
        campaign.gold = points.max(0);
    }

    Ok(SaveData {
        version: config.version.clone(),
        campaign,
    })
}

#[cfg(test)]
mod tests;
