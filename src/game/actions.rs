//! Interpretation of UI intents into session and campaign changes.

use super::Game;
use crate::state::{GameSession, GuardKind};
use crate::ui::UiAction;

impl Game {
    pub(super) fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::RequestNewCampaign => {
                // Only overwrite an existing save behind a confirmation prompt.
                if self.session.request_new_campaign(self.save_exists) {
                    self.start_new_campaign();
                }
            }
            UiAction::NewCampaign => {
                self.session.cancel_confirm();
                self.start_new_campaign();
            }
            UiAction::DismissConfirm => self.session.cancel_confirm(),
            UiAction::ContinueCampaign => {
                if self.save_exists {
                    self.load_game();
                } else {
                    self.notifications.warning("No saved campaign");
                }
            }
            UiAction::OpenMap => self.session.open_map(),
            UiAction::OpenLoadout => self.session.open_loadout(),
            UiAction::OpenShop => self.session.open_shop(),
            UiAction::OpenCarriages => self.session.open_carriages(),
            UiAction::OpenGuards => self.session.open_guards(),
            UiAction::OpenUpgrades => self.session.open_upgrades(),
            UiAction::OpenSettings => self.session.open_settings(),
            UiAction::OpenCodex => self.session.open_codex(),
            UiAction::SetCodexTab(tab) => self.session.set_codex_tab(tab),
            UiAction::ReturnTitle => self.session.return_title(),
            UiAction::PauseGame => self.session.pause_play(),
            UiAction::ResumeGame => self.session.resume_play(),
            UiAction::SelectMission(id) => self.session.select_mission(&id),
            UiAction::SelectRouteChoice(id) => {
                if self.session.select_route_choice(&self.data, &id) {
                    self.auto_save();
                }
            }
            UiAction::SelectGuard(id) => {
                self.session.select_guard(&id);
                self.auto_save();
            }
            UiAction::AssignGuardSlot(slot, id) => {
                self.session.assign_guard_slot(slot, &id);
                self.auto_save();
            }
            UiAction::ClearGuardSlot(slot) => {
                self.session.clear_guard_slot(slot);
                self.auto_save();
            }
            UiAction::AssignRangedSlot(slot, id) => {
                self.session.assign_ranged_slot(slot, &id);
                self.auto_save();
            }
            UiAction::ClearRangedSlot(slot) => {
                self.session.clear_ranged_slot(slot);
                self.auto_save();
            }
            UiAction::AssignEquipmentSlot(slot, id) => {
                self.session.campaign.assign_equipment_slot(slot, &id);
                self.auto_save();
            }
            UiAction::ClearEquipmentSlot(slot) => {
                self.session.campaign.clear_equipment_slot(slot);
                self.auto_save();
            }
            UiAction::HireGuard(id) => {
                let kind = GuardKind::from_id(&id);
                if self.session.hire_guard(&id) {
                    self.notifications
                        .success(format!("Hired {}", kind.label()));
                    self.auto_save();
                } else if !self.session.campaign.is_guard_unlocked(kind) {
                    self.notifications.warning("Guard locked");
                } else if self.session.campaign.is_guard_hired(kind) {
                    self.notifications.info("Guard already hired");
                } else {
                    self.notifications.warning("Not enough gold");
                }
            }
            UiAction::UpgradeGuardStar(id) => {
                let kind = GuardKind::from_id(&id);
                if self.session.upgrade_guard_star(&id) {
                    self.notifications.success(format!(
                        "{} reached {} star",
                        kind.label(),
                        self.session.campaign.guard_star_level(kind)
                    ));
                    self.auto_save();
                } else if self
                    .session
                    .campaign
                    .guard_star_upgrade_cost(kind)
                    .is_none()
                {
                    self.notifications.info("Guard already at 3 stars");
                } else {
                    self.notifications.warning("Not enough gold");
                }
            }
            UiAction::TreatGuard(id) => {
                let kind = GuardKind::from_id(&id);
                if self.session.treat_guard(&id) {
                    self.notifications
                        .success(format!("{} treated and back on duty", kind.label()));
                    self.auto_save();
                } else {
                    self.notifications.warning("Not enough gold to treat");
                }
            }
            UiAction::ToggleSetting(id) => {
                if self.session.toggle_setting(&id) {
                    self.notifications.info(format!(
                        "{} {}",
                        setting_label(&id),
                        setting_value(&self.session, &id)
                    ));
                    if let Err(err) = self.write_save() {
                        self.notifications
                            .warning(format!("Settings save failed: {}", err));
                    }
                }
            }
            UiAction::SetDifficulty(id) => {
                if self.session.set_difficulty(&id) {
                    self.notifications.info(format!(
                        "Difficulty: {}",
                        self.session.campaign.difficulty_preset.label()
                    ));
                    if let Err(err) = self.write_save() {
                        self.notifications
                            .warning(format!("Settings save failed: {}", err));
                    }
                }
            }
            UiAction::BeginMission => {
                if self.session.start_selected_mission(&self.data) {
                    if let Some(run) = &self.session.mission {
                        self.notifications
                            .info(format!("Route started: {}", run.mission_name));
                    }
                } else {
                    self.notifications.warning("Route locked");
                }
            }
            UiAction::OpenOutfitter => self.session.open_outfitter(),
            UiAction::OpenRecords => self.session.open_records(),
            UiAction::SelectStake(id) => {
                if self.session.select_stake(&id, &self.data) {
                    self.auto_save();
                }
            }
            UiAction::UnlockStartingRelic(id) => {
                if self.session.unlock_starting_relic(&id, &self.data) {
                    self.notifications.success("Starting relic unlocked");
                    self.auto_save();
                } else {
                    self.notifications.warning("Cannot unlock that relic");
                }
            }
            UiAction::StartExpedition => {
                // A fresh nonce per run so free expeditions actually vary.
                let nonce = macroquad::miniquad::date::now().to_bits();
                if self.session.start_journey(&self.data, nonce) {
                    self.notifications.info("Expedition begun — leg 1");
                } else {
                    self.notifications.warning("Could not start expedition");
                }
            }
            UiAction::StartDailyExpedition => {
                // Today's UTC day-number: the same seed for everyone today, so
                // the run is reproducible and shareable.
                let day = (macroquad::miniquad::date::now() / 86_400.0) as u64;
                let seed = day.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xDA11_0DA1_1000_u64;
                if self.session.start_journey_seeded(&self.data, seed, true) {
                    self.notifications.info("Daily expedition begun — leg 1");
                } else {
                    self.notifications.warning("Could not start expedition");
                }
            }
            UiAction::JourneyPressOn => {
                if self.session.journey_press_on(&self.data) {
                    if let Some(leg) = self.session.journey.as_ref().map(|j| j.leg) {
                        self.notifications.info(format!("Leg {} — set out", leg));
                    }
                }
            }
            UiAction::JourneyResolveEvent(index) => {
                let result = self
                    .session
                    .journey
                    .as_ref()
                    .and_then(|j| j.pending_event.as_ref())
                    .and_then(|id| self.data.run_events.get(id))
                    .and_then(|event| event.options.get(index))
                    .map(|option| option.result.clone());
                if self.session.journey_resolve_event(index, &self.data) {
                    if let Some(result) = result {
                        self.notifications.info(result);
                    }
                }
            }
            UiAction::JourneyBeginLeg(index) => {
                let title = self
                    .session
                    .journey
                    .as_ref()
                    .and_then(|j| j.pending_legs.as_ref())
                    .and_then(|legs| legs.get(index))
                    .map(|option| option.title(&self.data));
                if self.session.journey_begin_leg(index, &self.data) {
                    if let Some(title) = title {
                        self.notifications.info(format!("Set out: {}", title));
                    }
                }
            }
            UiAction::JourneyChooseReward(index) => {
                let reward = self
                    .session
                    .journey
                    .as_ref()
                    .and_then(|j| j.pending_rewards.as_ref())
                    .and_then(|r| r.get(index))
                    .cloned();
                if self.session.journey_choose_reward(index, &self.data) {
                    if let Some(reward) = reward {
                        self.notifications
                            .success(format!("{} taken", reward.title(&self.data)));
                    }
                }
            }
            UiAction::JourneyRepair => {
                if self.session.journey_repair() {
                    self.notifications.success("Carriage repaired");
                } else {
                    self.notifications.warning("Cannot afford repairs");
                }
            }
            UiAction::JourneyBank => {
                let banked = self
                    .session
                    .journey
                    .as_ref()
                    .filter(|j| j.alive)
                    .map(|j| j.banked_gold);
                self.session.journey_bank_and_return();
                if let Some(gold) = banked {
                    self.notifications.success(format!("Banked {} gold", gold));
                }
                self.auto_save();
            }
            UiAction::RetryMission => {
                if !self.session.retry_result_mission(&self.data) {
                    self.notifications.warning("Could not restart route");
                }
            }
            UiAction::UseRepair => {
                if self.session.use_repair() {
                    self.notifications.success("Emergency repair used");
                }
            }
            UiAction::BuyUpgrade(id) => {
                let Some(upgrade) = self.data.upgrades.get(&id) else {
                    self.notifications
                        .warning(format!("Unknown upgrade: {}", id));
                    return;
                };
                if self.session.buy_upgrade(upgrade) {
                    self.notifications
                        .success(format!("Upgraded {}", upgrade.name));
                    self.auto_save();
                } else {
                    self.notifications.warning("Not enough gold");
                }
            }
            UiAction::BuyChassis(id) => {
                if self.session.buy_chassis(&self.data, &id) {
                    let name = self
                        .data
                        .chassis
                        .get(&id)
                        .map(|chassis| chassis.name.clone())
                        .unwrap_or(id);
                    self.notifications.success(format!("Bought {}", name));
                    self.auto_save();
                } else {
                    self.notifications.warning("Cannot buy that carriage");
                }
            }
            UiAction::SelectChassis(id) => {
                if self.session.select_chassis(&self.data, &id) {
                    let name = self
                        .data
                        .chassis
                        .get(&id)
                        .map(|chassis| chassis.name.clone())
                        .unwrap_or(id);
                    self.notifications.info(format!("Now driving the {}", name));
                    self.auto_save();
                }
            }
            UiAction::SelectFrame(id) => {
                if self.session.select_frame(&self.data, &id) {
                    let name = self
                        .data
                        .carriage_frames
                        .get(&id)
                        .map(|frame| frame.name.clone())
                        .unwrap_or(id);
                    self.notifications.info(format!("Frame set: {}", name));
                    self.auto_save();
                }
            }
            UiAction::BuyReinforcedKit => {
                if self.session.buy_reinforced_kit() {
                    self.notifications.success(format!(
                        "Reinforced Kit bought ({} in stock)",
                        self.session.campaign.reinforced_kits
                    ));
                    self.auto_save();
                } else {
                    self.notifications.warning("Not enough gold");
                }
            }
            UiAction::Save => self.save_game(),
            UiAction::Load => self.load_game(),
            UiAction::ExitGame => macroquad::miniquad::window::quit(),
        }
    }
}

fn setting_label(id: &str) -> &'static str {
    match id {
        "route_motion" => "Route motion",
        "alerts" => "Route alerts",
        "auto_save" => "Autosave",
        "generous_timers" => "Generous timers",
        "slower_waves" => "Slower waves",
        "sturdy_carriage" => "Sturdy carriage",
        _ => "Setting",
    }
}

fn setting_value(session: &GameSession, id: &str) -> &'static str {
    let enabled = match id {
        "route_motion" => session.campaign.route_motion_enabled,
        "alerts" => session.campaign.alerts_enabled,
        "auto_save" => session.campaign.auto_save_enabled,
        "generous_timers" => session.campaign.generous_timers,
        "slower_waves" => session.campaign.slower_waves,
        "sturdy_carriage" => session.campaign.sturdy_carriage,
        _ => false,
    };
    if enabled {
        "on"
    } else {
        "off"
    }
}
