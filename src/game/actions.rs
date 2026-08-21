//! Interpretation of UI intents into session and campaign changes.

use super::Game;
use crate::state::GuardKind;
use crate::ui::UiAction;

impl Game {
    pub(super) fn apply_action(&mut self, action: UiAction) {
        self.audio
            .ui(crate::audio::AudioCue::UiConfirm, &self.settings);
        if self.apply_navigation_action(&action)
            || self.apply_settings_action(&action)
            || self.apply_expedition_action(&action)
        {
            return;
        }
        match action {
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
            UiAction::PurchaseGuardSpecialization(id) => {
                if self.session.purchase_guard_specialization(&self.data, &id) {
                    self.notifications.success("Guard specialization learned");
                    self.auto_save();
                } else {
                    self.notifications
                        .warning("Requires a 3-star guard and enough gold");
                }
            }
            UiAction::BuyCosmetic(id) => {
                if self.session.buy_cosmetic(&self.data, &id) {
                    self.notifications.success("Cosmetic unlocked");
                    self.auto_save();
                } else {
                    self.notifications
                        .warning("Not enough gold or already owned");
                }
            }
            UiAction::SelectCosmetic(id) => {
                if self.session.select_cosmetic(&self.data, &id) {
                    self.notifications.info("Convoy colors updated");
                    self.auto_save();
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
                let _ = self.session.request_buy_chassis(&self.data, &id);
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
            UiAction::SelectSaveSlot(slot) => match self.select_save_slot(&slot) {
                Ok(()) => self.notifications.info(format!("Active save slot: {slot}")),
                Err(err) => self
                    .notifications
                    .warning(format!("Slot load failed: {err}")),
            },
            UiAction::CreateSaveSlot(slot) => match self.create_save_slot(&slot) {
                Ok(()) => self
                    .notifications
                    .success(format!("Created save slot: {slot}")),
                Err(err) => self
                    .notifications
                    .warning(format!("Slot create failed: {err}")),
            },
            UiAction::RenameSaveSlot(slot) => match self.rename_active_save_slot(&slot) {
                Ok(()) => self
                    .notifications
                    .success(format!("Save slot renamed to {slot}")),
                Err(err) => self
                    .notifications
                    .warning(format!("Slot rename failed: {err}")),
            },
            UiAction::DeleteSaveSlot => match self.delete_active_save_slot() {
                Ok(()) => self.notifications.info("Save slot deleted"),
                Err(err) => self
                    .notifications
                    .warning(format!("Slot delete failed: {err}")),
            },
            _ => {}
        }
    }
}
