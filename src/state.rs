//! Campaign state, save data, and screen-level mission orchestration.

mod campaign;
mod chassis;
mod entities;
mod equipment;
mod journey;
mod mission;

pub use entities::*;
pub use equipment::*;
pub use journey::Journey;
pub use mission::{MissionInput, MissionReport, MissionRun};

use crate::data::{GameConfig, GameData, UpgradeDef};
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
}

impl GameSession {
    pub fn new(config: &GameConfig, first_mission_id: Option<&str>) -> Self {
        Self {
            campaign: CampaignState::new(config, first_mission_id),
            screen: Screen::Title,
            mission: None,
            result: None,
            journey: None,
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
        }
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
            _ => return false,
        }
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

        self.mission = Some(MissionRun::new(mission, &self.campaign));
        self.result = None;
        self.screen = Screen::Playing;
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
                self.resolve_journey_leg(&report);
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
