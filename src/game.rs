//! High-level game loop, state transitions, and toolkit integration.

mod actions;
mod capture;
mod persistence;

use crate::audio::GameAudio;
use crate::data::GameData;
use crate::localization::{font_fallbacks, Language, Localizer};
use crate::settings::RuntimeSettings;
use crate::state::{GameSession, MissionInput, Screen};
use crate::ui::{self, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::input::{GamepadFrame, GamepadInput};
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::AutoSaveManager;
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame};
use macroquad_toolkit::ui::virtual_mouse_position;
use macroquad_toolkit::ui::HoverTooltip;
use std::cell::RefCell;

pub struct Game {
    data: GameData,
    session: GameSession,
    assets: AssetManager,
    notifications: NotificationManager,
    events: EventBus<UiAction>,
    save_exists: bool,
    save_slots: Vec<String>,
    gamepad: GamepadInput,
    controller_connected: bool,
    pub(crate) settings: RuntimeSettings,
    audio: GameAudio,
    localizer: Localizer,
    autosave: AutoSaveManager,
    save_dirty: bool,
    startup_error: Option<String>,
    tooltip: RefCell<HoverTooltip>,
}

impl Game {
    pub async fn new() -> Self {
        let (data, startup_error) = match GameData::load() {
            Ok(data) => (data, None),
            Err(err) => (GameData::recovery(&err), Some(err)),
        };

        // Surface mission-data typos immediately in dev/CI builds; release keeps
        // the tolerant spawn-time fallback rather than crashing a player.
        let startup_error = validate_startup_data(&data, startup_error);

        let mut assets = AssetManager::new();
        let placeholder = Image::gen_image_color(16, 16, Color::new(0.8, 0.2, 0.5, 1.0));
        assets.set_placeholder_texture_direct(Texture2D::from_image(&placeholder));
        let loaded_assets = assets.load_texture_configs(&data.texture_manifest).await;

        let mut notifications = NotificationManager::new();
        notifications.info(format!(
            "Carriage Run ready; {} manifest textures loaded",
            loaded_assets
        ));

        let settings = RuntimeSettings::load(&data.config.game_name);
        settings.apply();
        let mut localizer = Localizer::load(Language::from_id(&settings.language))
            .unwrap_or_else(|_| Localizer::english());
        let locale = localizer.language();
        let layout_warnings = localizer.layout_warnings();
        let _ = localizer.text("menu.new_campaign");
        let missing_keys = localizer.missing_keys().count();
        notifications.info(format!(
            "Language {} ready; {} fallback font(s)",
            locale.id(),
            font_fallbacks(locale).len()
        ));
        if !layout_warnings.is_empty() {
            notifications.warning(format!(
                "{} localized string(s) may need a wider layout",
                layout_warnings.len()
            ));
        }
        if missing_keys > 0 {
            notifications.warning(format!("{} localization key(s) missing", missing_keys));
        }
        let mut session = GameSession::new(&data.config, data.first_mission_id());
        session.sync_chassis(&data);
        let mut audio = GameAudio::new();
        audio.load_generated().await;
        let mut game = Self {
            data,
            session,
            assets,
            notifications,
            events: EventBus::new(),
            save_exists: false,
            save_slots: Vec::new(),
            gamepad: GamepadInput::new(),
            controller_connected: false,
            settings,
            audio,
            localizer,
            autosave: AutoSaveManager::new(30.0),
            save_dirty: false,
            startup_error,
            tooltip: RefCell::new(HoverTooltip::new()),
        };
        if let Some(error) = &game.startup_error {
            game.notifications
                .danger(format!("Caravan data needs attention: {error}"));
            return game;
        }
        game.audio.apply_settings(&game.settings);
        game.refresh_save_state();
        // A corrupt save otherwise leaves "Continue" offered but broken: gate it
        // on the save actually loading, and tell the player it was skipped.
        if game.save_exists && game.try_load_save().is_err() {
            let slot = game.session.campaign.active_save_slot.clone();
            if let Ok(quarantined) =
                macroquad_toolkit::persistence::quarantine_slot(&game.data.config.game_name, &slot)
            {
                game.notifications
                    .warning(format!("Damaged save moved aside as {quarantined}"));
            }
            let recovered = (1..=3).find_map(|index| {
                game.try_load_save_from_slot(&format!("{slot}_backup_{index}"))
                    .ok()
            });
            if let Some(save) = recovered {
                game.session = GameSession::from_save(save, game.data.first_mission_id());
                game.session.sync_chassis(&game.data);
                game.save_dirty = true;
                game.notifications
                    .warning("Primary save was damaged — restored the newest backup");
            } else {
                game.save_exists = false;
                game.notifications
                    .warning("Saved campaign is unreadable — starting fresh");
            }
        }
        game
    }

    pub fn update(&mut self, dt: f32) {
        self.notifications.update(dt);
        if self.startup_error.is_some() {
            return;
        }
        self.audio.set_screen(self.session.screen);
        self.audio.apply_settings(&self.settings);
        self.audio.set_page_focused(true, &self.settings);
        self.handle_global_keys();
        self.apply_pending_actions();
        if self.save_dirty && self.session.campaign.auto_save_enabled {
            if let Ok(true) = self.autosave.update(dt, true, || Ok(())) {
                if let Err(err) = self.write_save() {
                    self.notifications
                        .warning(format!("Autosave failed: {}", err));
                }
            }
        } else {
            self.autosave.reset_timer();
        }

        let mouse = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let pad = self.gamepad.capture();
        self.controller_connected = pad.connected;
        self.handle_gamepad(pad);
        let touch_down = is_mouse_button_down(MouseButton::Left);
        let input = MissionInput {
            mouse,
            pressed: is_mouse_button_pressed(MouseButton::Left),
            down: is_mouse_button_down(MouseButton::Left),
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
        };

        if let Some(report) = self.session.update_play(&self.data, dt, input) {
            if report.success {
                self.notifications.success(format!(
                    "{} complete: {} gold",
                    report.mission_name, report.reward
                ));
            } else {
                self.notifications.warning(report.reason.clone());
            }
            self.auto_save();
            self.audio.combat(
                if report.success {
                    crate::audio::AudioCue::Victory
                } else {
                    crate::audio::AudioCue::Defeat
                },
                1.0,
                &self.settings,
            );
        }
    }

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);

        if let Some(error) = &self.startup_error {
            ui::draw_recovery_screen(error);
            self.notifications
                .draw_with_config(&NotificationRenderConfig {
                    anchor: NotificationAnchor::BottomRight,
                    ..Default::default()
                });
            return;
        }

        let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let ctx = UiContext {
            data: &self.data,
            session: &self.session,
            assets: &self.assets,
            save_exists: self.save_exists,
            loaded_assets: self.assets.len(),
            ui: &virtual_ui,
            settings: &self.settings,
            localization: &self.localizer,
            save_slots: &self.save_slots,
            active_save_slot: &self.session.campaign.active_save_slot,
            tooltip: &self.tooltip,
            controller_connected: self.controller_connected,
        };

        let actions = ui::draw_game_ui(ctx);
        end_virtual_ui_frame();
        self.tooltip.borrow_mut().draw(
            &macroquad_toolkit::ui::TooltipStyle::default(),
            None,
            get_time(),
        );

        for action in actions {
            self.events.push(action);
        }

        self.notifications
            .draw_with_config(&NotificationRenderConfig {
                anchor: NotificationAnchor::BottomRight,
                ..Default::default()
            });
    }

    fn handle_global_keys(&mut self) {
        if is_key_pressed(self.settings.bindings.key("save")) {
            self.events.push(UiAction::Save);
        }
        if is_key_pressed(self.settings.bindings.key("load")) {
            self.events.push(UiAction::Load);
        }
        if is_key_pressed(KeyCode::Escape) {
            // A confirmation dialog swallows Escape as a cancel, whatever screen
            // it is layered over.
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
                // Expedition decisions must be made with the on-screen buttons
                // so a run is never abandoned by an accidental keypress.
                Screen::Journey => {}
                Screen::Title => {}
            }
        }
    }

    fn handle_gamepad(&mut self, pad: GamepadFrame) {
        if !pad.connected {
            return;
        }
        if self.session.pending_confirm.is_some() {
            if pad.cancel {
                self.events.push(UiAction::DismissConfirm);
            } else if pad.confirm {
                match self.session.pending_confirm.clone() {
                    Some(crate::state::ConfirmPrompt::NewCampaign) => {
                        self.events.push(UiAction::NewCampaign)
                    }
                    Some(crate::state::ConfirmPrompt::BuyChassis(id)) => {
                        self.events.push(UiAction::ConfirmBuyChassis(id))
                    }
                    Some(crate::state::ConfirmPrompt::AbandonExpedition) => {
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
            Screen::Title => {
                self.events.push(if self.save_exists {
                    UiAction::ContinueCampaign
                } else {
                    UiAction::RequestNewCampaign
                });
            }
            Screen::MissionMap => self.events.push(UiAction::OpenLoadout),
            Screen::Loadout => self.events.push(UiAction::BeginMission),
            Screen::Paused => self.events.push(UiAction::ResumeGame),
            Screen::Results => self.events.push(UiAction::OpenMap),
            Screen::Settings => {
                self.events.push(if self.session.mission.is_some() {
                    UiAction::ResumeGame
                } else {
                    UiAction::OpenMap
                });
            }
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

    fn apply_pending_actions(&mut self) {
        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
    }
}

#[cfg(debug_assertions)]
fn validate_startup_data(data: &GameData, mut startup_error: Option<String>) -> Option<String> {
    let missions = data.missions_ordered();
    if startup_error.is_none() {
        for result in [
            crate::state::validate_mission_content(&missions),
            crate::state::validate_mission_reachability(&missions),
            crate::state::validate_campaign_metadata(&missions),
        ] {
            if let Err(err) = result {
                startup_error = Some(err);
                break;
            }
        }
    }
    startup_error
}

#[cfg(not(debug_assertions))]
fn validate_startup_data(_data: &GameData, startup_error: Option<String>) -> Option<String> {
    startup_error
}
