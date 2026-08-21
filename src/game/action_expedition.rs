//! Expedition entry, branch, event, and reward intents.

use super::Game;
use crate::ui::UiAction;

impl Game {
    pub(super) fn apply_expedition_action(&mut self, action: &UiAction) -> bool {
        match action {
            UiAction::SelectStake(id) => {
                if self.session.select_stake(id, &self.data) {
                    self.auto_save();
                }
            }
            UiAction::UnlockStartingRelic(id) => {
                if self.session.unlock_starting_relic(id, &self.data) {
                    self.notifications.success("Starting relic unlocked");
                    self.auto_save();
                } else {
                    self.notifications.warning("Cannot unlock that relic");
                }
            }
            UiAction::ToggleStartingRelic(id) => {
                if self.session.toggle_starting_relic(id, &self.data) {
                    self.notifications.info("Starting relic loadout updated");
                    self.auto_save();
                } else {
                    self.notifications
                        .warning("Both starting relic slots are full");
                }
            }
            UiAction::StartExpedition => {
                let nonce = macroquad::miniquad::date::now().to_bits();
                if self.session.start_journey(&self.data, nonce) {
                    self.notifications.info("Expedition begun — leg 1");
                } else {
                    self.notifications.warning("Could not start expedition");
                }
            }
            UiAction::StartDailyExpedition => {
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
                    if let Some(leg) = self.session.journey.as_ref().map(|journey| journey.leg) {
                        self.notifications.info(format!("Leg {} — set out", leg));
                    }
                }
            }
            UiAction::JourneyResolveEvent(index) => {
                let result = self
                    .session
                    .journey
                    .as_ref()
                    .and_then(|journey| journey.pending_event.as_ref())
                    .and_then(|id| self.data.run_events.get(id))
                    .and_then(|event| event.options.get(*index))
                    .map(|option| option.result.clone());
                if self.session.journey_resolve_event(*index, &self.data) {
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
                    .and_then(|journey| journey.pending_legs.as_ref())
                    .and_then(|legs| legs.get(*index))
                    .map(|option| option.title(&self.data));
                if self.session.journey_begin_leg(*index, &self.data) {
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
                    .and_then(|journey| journey.pending_rewards.as_ref())
                    .and_then(|rewards| rewards.get(*index))
                    .cloned();
                if self.session.journey_choose_reward(*index, &self.data) {
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
                    .filter(|journey| journey.alive)
                    .map(|journey| journey.banked_gold);
                self.session.journey_bank_and_return();
                if let Some(gold) = banked {
                    self.notifications.success(format!("Banked {} gold", gold));
                }
                self.auto_save();
            }
            _ => return false,
        }
        true
    }
}
