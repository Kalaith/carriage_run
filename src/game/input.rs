//! Keyboard, gamepad, and touch input translation for the game shell.

use super::Game;
use crate::state::{ConfirmPrompt, MissionInput, Screen};
use crate::ui::{self, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::input::GamepadFrame;
use macroquad_toolkit::ui::virtual_mouse_position;

impl Game {
    pub(super) fn capture_mission_input(&self, pad: GamepadFrame) -> MissionInput {
        let mouse = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let touch_down = is_mouse_button_down(MouseButton::Left);
        MissionInput {
            mouse,
            pressed: is_mouse_button_pressed(MouseButton::Left),
            down: touch_down,
            released: is_mouse_button_released(MouseButton::Left),
            repair_pressed: is_key_pressed(self.settings.bindings.key("repair")),
            play_rect: ui::play_rect(),
            steer_left: is_key_down(self.settings.bindings.key("steer_left"))
                || is_key_down(KeyCode::Left)
                || pad.left
                || (touch_down && ui::touch_steer_left_rect().contains(mouse)),
            steer_right: is_key_down(self.settings.bindings.key("steer_right"))
                || is_key_down(KeyCode::Right)
                || pad.right
                || (touch_down && ui::touch_steer_right_rect().contains(mouse)),
            boost: is_key_down(self.settings.bindings.key("boost"))
                || is_key_down(KeyCode::Up)
                || pad.up
                || (touch_down && ui::touch_boost_rect().contains(mouse)),
            brake: is_key_down(self.settings.bindings.key("brake"))
                || is_key_down(KeyCode::Down)
                || pad.down
                || (touch_down && ui::touch_brake_rect().contains(mouse)),
        }
    }

    pub(super) fn handle_global_keys(&mut self) {
        if is_key_pressed(self.settings.bindings.key("save")) {
            self.events.push(UiAction::Save);
        }
        if is_key_pressed(self.settings.bindings.key("load")) {
            self.events.push(UiAction::Load);
        }
        if is_key_pressed(KeyCode::Escape) {
            if self.session.pending_confirm.is_some() {
                self.events.push(UiAction::DismissConfirm);
                return;
            }
            match self.session.screen {
                Screen::Playing => self.events.push(UiAction::PauseGame),
                Screen::Paused => self.events.push(UiAction::ResumeGame),
                Screen::Results => self.events.push(UiAction::OpenMap),
                Screen::Settings if self.session.mission.is_some() => {
                    self.events.push(UiAction::ResumeGame)
                }
                Screen::Loadout
                | Screen::Shop
                | Screen::Carriages
                | Screen::Guards
                | Screen::Upgrades
                | Screen::Settings => self.events.push(UiAction::OpenMap),
                Screen::Outfitter => self.events.push(UiAction::OpenLoadout),
                Screen::Records => self.events.push(UiAction::OpenOutfitter),
                Screen::MissionMap => self.events.push(UiAction::ReturnTitle),
                Screen::Codex => self.events.push(UiAction::ReturnTitle),
                Screen::Cosmetics | Screen::Credits => self.events.push(UiAction::ReturnTitle),
                Screen::Journey | Screen::Title => {}
            }
        }
    }

    pub(super) fn handle_gamepad(&mut self, pad: GamepadFrame) {
        if !pad.connected {
            return;
        }
        if self.session.pending_confirm.is_some() {
            if pad.cancel {
                self.events.push(UiAction::DismissConfirm);
            } else if pad.confirm {
                match self.session.pending_confirm.clone() {
                    Some(ConfirmPrompt::NewCampaign) => self.events.push(UiAction::NewCampaign),
                    Some(ConfirmPrompt::BuyChassis(id)) => {
                        self.events.push(UiAction::ConfirmBuyChassis(id))
                    }
                    Some(ConfirmPrompt::AbandonExpedition) => {
                        self.events.push(UiAction::AbandonExpedition)
                    }
                    None => {}
                }
            }
            return;
        }

        if pad.menu {
            match self.session.screen {
                Screen::Playing => self.events.push(UiAction::PauseGame),
                Screen::Paused => self.events.push(UiAction::ResumeGame),
                _ => self.events.push(UiAction::OpenSettings),
            }
            return;
        }
        if pad.cancel {
            match self.session.screen {
                Screen::Playing => self.events.push(UiAction::PauseGame),
                Screen::Paused => self.events.push(UiAction::ResumeGame),
                Screen::Results => self.events.push(UiAction::OpenMap),
                Screen::Settings if self.session.mission.is_some() => {
                    self.events.push(UiAction::ResumeGame)
                }
                Screen::Loadout
                | Screen::Shop
                | Screen::Carriages
                | Screen::Guards
                | Screen::Upgrades
                | Screen::Settings => self.events.push(UiAction::OpenMap),
                Screen::Outfitter => self.events.push(UiAction::OpenLoadout),
                Screen::Records => self.events.push(UiAction::OpenOutfitter),
                Screen::MissionMap | Screen::Codex | Screen::Cosmetics | Screen::Credits => {
                    self.events.push(UiAction::ReturnTitle)
                }
                Screen::Journey | Screen::Title => {}
            }
            return;
        }
        if pad.secondary && self.session.screen == Screen::Playing {
            if let Some(run) = &mut self.session.mission {
                run.cycle_first_guard_order();
                self.notifications
                    .info("Controller order: first guard stance cycled");
            }
            return;
        }
        if !pad.confirm {
            return;
        }
        match self.session.screen {
            Screen::Title => self.events.push(if self.save_exists {
                UiAction::ContinueCampaign
            } else {
                UiAction::RequestNewCampaign
            }),
            Screen::MissionMap => self.events.push(UiAction::OpenLoadout),
            Screen::Loadout => self.events.push(UiAction::BeginMission),
            Screen::Paused => self.events.push(UiAction::ResumeGame),
            Screen::Results => self.events.push(UiAction::OpenMap),
            Screen::Settings => self.events.push(if self.session.mission.is_some() {
                UiAction::ResumeGame
            } else {
                UiAction::OpenMap
            }),
            Screen::Outfitter => self.events.push(UiAction::OpenLoadout),
            Screen::Records => self.events.push(UiAction::OpenOutfitter),
            Screen::Codex | Screen::Cosmetics | Screen::Credits => {
                self.events.push(UiAction::ReturnTitle)
            }
            Screen::Playing
            | Screen::Shop
            | Screen::Carriages
            | Screen::Guards
            | Screen::Upgrades
            | Screen::Journey => {}
        }
    }
}
