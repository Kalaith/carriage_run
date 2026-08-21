//! High-level game loop, state transitions, and toolkit integration.

mod action_expedition;
mod action_navigation;
mod action_settings;
mod actions;
mod capture;
mod input;
mod persistence;

use crate::audio::GameAudio;
use crate::data::GameData;
use crate::localization::{font_fallbacks, Language, Localizer};
use crate::settings::RuntimeSettings;
use crate::state::GameSession;
use crate::ui::{self, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::input::GamepadInput;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::AutoSaveManager;
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame};
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

        let pad = self.gamepad.capture();
        self.controller_connected = pad.connected;
        self.handle_gamepad(pad);
        let input = self.capture_mission_input(pad);

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
