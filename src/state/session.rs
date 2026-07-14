//! Live game session: screen navigation, roster edits, and mission dispatch.

use super::{
    default_guard_id, CampaignState, CodexTab, ConfirmPrompt, DifficultyPreset, GuardKind, Journey,
    MissionInput, MissionRecord, MissionReport, MissionRun, SaveData, Screen, REINFORCED_KIT_COST,
};
use crate::data::{GameConfig, GameData, UpgradeDef};

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

    pub(super) fn apply_report(&mut self, report: MissionReport) {
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
