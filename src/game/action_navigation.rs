//! Navigation and confirmation intents from the UI.

use super::Game;
use crate::ui::UiAction;

impl Game {
    pub(super) fn apply_navigation_action(&mut self, action: &UiAction) -> bool {
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
            UiAction::ConfirmBuyChassis(id) => {
                self.session.cancel_confirm();
                if self.session.buy_chassis(&self.data, id) {
                    self.notifications.success("Chassis purchased");
                    self.auto_save();
                }
            }
            UiAction::RequestAbandonExpedition => self.session.request_abandon_expedition(),
            UiAction::AbandonExpedition => {
                if self.session.abandon_expedition() {
                    self.notifications
                        .info("Expedition abandoned; campaign gold is safe");
                    self.auto_save();
                }
            }
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
            UiAction::OpenCosmetics => self.session.open_cosmetics(),
            UiAction::OpenCredits => self.session.open_credits(),
            UiAction::OpenOutfitter => self.session.open_outfitter(),
            UiAction::OpenRecords => self.session.open_records(),
            UiAction::SetCodexTab(tab) => self.session.set_codex_tab(*tab),
            UiAction::ReturnTitle => self.session.return_title(),
            UiAction::PauseGame => self.session.pause_play(),
            UiAction::ResumeGame => self.session.resume_play(),
            UiAction::ExitGame => macroquad::miniquad::window::quit(),
            _ => return false,
        }
        true
    }
}
