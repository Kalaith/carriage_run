//! High-level game loop, state transitions, and toolkit integration.

mod actions;
mod capture;
mod persistence;

use crate::data::GameData;
use crate::state::{GameSession, MissionInput, Screen};
use crate::ui::{self, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame};
use macroquad_toolkit::ui::virtual_mouse_position;

#[cfg(target_arch = "wasm32")]
const ASSET_PACK_PATH: &str = "assets.zip?v=20260615-title-art";
#[cfg(not(target_arch = "wasm32"))]
const ASSET_PACK_PATH: &str = "assets.zip";

pub struct Game {
    data: GameData,
    session: GameSession,
    assets: AssetManager,
    notifications: NotificationManager,
    events: EventBus<UiAction>,
    save_exists: bool,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load().unwrap_or_else(|err| {
            panic!("Carriage Run embedded data failed to load: {}", err);
        });

        // Surface mission-data typos immediately in dev/CI builds; release keeps
        // the tolerant spawn-time fallback rather than crashing a player.
        #[cfg(debug_assertions)]
        {
            let missions = data.missions_ordered();
            if let Err(err) = crate::state::validate_mission_content(&missions) {
                panic!("Carriage Run mission data invalid: {}", err);
            }
            if let Err(err) = crate::state::validate_mission_reachability(&missions) {
                panic!("Carriage Run mission graph invalid: {}", err);
            }
        }

        let mut assets = AssetManager::new();
        let placeholder = Image::gen_image_color(16, 16, Color::new(0.8, 0.2, 0.5, 1.0));
        assets.set_placeholder_texture_direct(Texture2D::from_image(&placeholder));
        let _ = assets.load_asset_pack(ASSET_PACK_PATH).await;
        let loaded_assets = assets.load_texture_configs(&data.texture_manifest).await;

        let mut notifications = NotificationManager::new();
        notifications.info(format!(
            "Carriage Run ready; {} manifest textures loaded",
            loaded_assets
        ));

        let mut session = GameSession::new(&data.config, data.first_mission_id());
        session.sync_chassis(&data);
        let mut game = Self {
            data,
            session,
            assets,
            notifications,
            events: EventBus::new(),
            save_exists: false,
        };
        game.refresh_save_state();
        // A corrupt save otherwise leaves "Continue" offered but broken: gate it
        // on the save actually loading, and tell the player it was skipped.
        if game.save_exists && game.try_load_save().is_err() {
            game.save_exists = false;
            game.notifications
                .warning("Saved campaign is unreadable — starting fresh");
        }
        game
    }

    pub fn update(&mut self, dt: f32) {
        self.notifications.update(dt);
        self.handle_global_keys();
        self.apply_pending_actions();

        let mouse = virtual_mouse_position(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let input = MissionInput {
            mouse,
            pressed: is_mouse_button_pressed(MouseButton::Left),
            down: is_mouse_button_down(MouseButton::Left),
            released: is_mouse_button_released(MouseButton::Left),
            repair_pressed: is_key_pressed(KeyCode::R),
            play_rect: ui::play_rect(),
            steer_left: is_key_down(KeyCode::A) || is_key_down(KeyCode::Left),
            steer_right: is_key_down(KeyCode::D) || is_key_down(KeyCode::Right),
            boost: is_key_down(KeyCode::Up) || is_key_down(KeyCode::Space),
            brake: is_key_down(KeyCode::Down) || is_key_down(KeyCode::LeftShift),
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
        }
    }

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);

        let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let ctx = UiContext {
            data: &self.data,
            session: &self.session,
            assets: &self.assets,
            save_exists: self.save_exists,
            loaded_assets: self.assets.len(),
            ui: &virtual_ui,
        };

        let actions = ui::draw_game_ui(ctx);
        end_virtual_ui_frame();

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
        if is_key_pressed(KeyCode::S) {
            self.events.push(UiAction::Save);
        }
        if is_key_pressed(KeyCode::L) {
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
                // Expedition decisions must be made with the on-screen buttons
                // so a run is never abandoned by an accidental keypress.
                Screen::Journey => {}
                Screen::Title => {}
            }
        }
    }

    fn apply_pending_actions(&mut self) {
        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
    }
}
