//! High-level game loop, state transitions, and toolkit integration.

use crate::data::GameData;
use crate::state::{migrate_save_value, GameSession, GuardKind, MissionInput, SaveData, Screen};
use crate::ui::{self, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{
    get_save_slots, load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
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
    save_slots: Vec<String>,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load().unwrap_or_else(|err| {
            panic!("Carriage Run embedded data failed to load: {}", err);
        });

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

        let session = GameSession::new(&data.config, data.first_mission_id());
        let mut game = Self {
            data,
            session,
            assets,
            notifications,
            events: EventBus::new(),
            save_exists: false,
            save_slots: Vec::new(),
        };
        game.refresh_save_state();
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
            match self.session.screen {
                Screen::Playing => self.events.push(UiAction::PauseGame),
                Screen::Paused => self.events.push(UiAction::ResumeGame),
                Screen::Results => self.events.push(UiAction::OpenMap),
                Screen::Settings if self.session.mission.is_some() => {
                    self.events.push(UiAction::ResumeGame)
                }
                Screen::Loadout
                | Screen::Shop
                | Screen::Guards
                | Screen::Upgrades
                | Screen::Settings => self.events.push(UiAction::OpenMap),
                Screen::MissionMap => self.events.push(UiAction::ReturnTitle),
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

    fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::NewCampaign => {
                self.session = GameSession::new(&self.data.config, self.data.first_mission_id());
                self.session.open_map();
                self.notifications.info("New caravan charter started");
                self.auto_save();
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
            UiAction::OpenGuards => self.session.open_guards(),
            UiAction::OpenUpgrades => self.session.open_upgrades(),
            UiAction::OpenSettings => self.session.open_settings(),
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
            UiAction::Save => self.save_game(),
            UiAction::Load => self.load_game(),
            UiAction::ExitGame => macroquad::miniquad::window::quit(),
        }
    }

    fn save_game(&mut self) {
        match self.write_save() {
            Ok(()) => self.notifications.success("Campaign saved"),
            Err(err) => self.notifications.danger(format!("Save failed: {}", err)),
        }
    }

    fn auto_save(&mut self) {
        if !self.session.campaign.auto_save_enabled {
            return;
        }

        if let Err(err) = self.write_save() {
            self.notifications
                .warning(format!("Autosave failed: {}", err));
        }
    }

    fn write_save(&mut self) -> Result<(), String> {
        let save = self.session.to_save(&self.data.config.version);
        save_to_slot_with_version(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &save,
            &self.data.config.version,
        )?;
        self.refresh_save_state();
        Ok(())
    }

    fn load_game(&mut self) {
        let first_mission = self.data.first_mission_id().map(ToOwned::to_owned);
        let loaded: Result<SaveData, String> = load_from_slot_with_migration(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &self.data.config.version,
            |version, value| {
                migrate_save_value(version, value, &self.data.config, first_mission.as_deref())
            },
        );

        match loaded {
            Ok(save) => {
                self.session = GameSession::from_save(save, self.data.first_mission_id());
                self.notifications.success("Campaign loaded");
                self.refresh_save_state();
            }
            Err(err) => self.notifications.warning(format!("Load failed: {}", err)),
        }
    }

    fn refresh_save_state(&mut self) {
        self.save_exists = slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
        self.save_slots = get_save_slots(&self.data.config.game_name);
    }
}

fn setting_label(id: &str) -> &'static str {
    match id {
        "route_motion" => "Route motion",
        "alerts" => "Route alerts",
        "auto_save" => "Autosave",
        _ => "Setting",
    }
}

fn setting_value(session: &GameSession, id: &str) -> &'static str {
    let enabled = match id {
        "route_motion" => session.campaign.route_motion_enabled,
        "alerts" => session.campaign.alerts_enabled,
        "auto_save" => session.campaign.auto_save_enabled,
        _ => false,
    };
    if enabled {
        "on"
    } else {
        "off"
    }
}
