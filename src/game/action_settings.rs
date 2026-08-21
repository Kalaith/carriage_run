//! Campaign difficulty and runtime preference intents.

use super::Game;
use crate::state::GameSession;
use crate::ui::UiAction;

impl Game {
    pub(super) fn apply_settings_action(&mut self, action: &UiAction) -> bool {
        match action {
            UiAction::ToggleSetting(id) => {
                if self.session.toggle_setting(id) {
                    self.notifications.info(format!(
                        "{} {}",
                        setting_label(id),
                        setting_value(&self.session, id)
                    ));
                    if let Err(err) = self.write_save() {
                        self.notifications
                            .warning(format!("Settings save failed: {}", err));
                    }
                }
            }
            UiAction::SetDifficulty(id) => {
                if self.session.set_difficulty(id) {
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
            UiAction::ToggleRuntimeSetting(id) => self.toggle_runtime_setting(id),
            UiAction::CycleRuntimeSetting(id) => self.cycle_runtime_setting(id),
            _ => return false,
        }
        true
    }

    fn toggle_runtime_setting(&mut self, id: &str) {
        match id {
            "fullscreen" => self.settings.display.fullscreen = !self.settings.display.fullscreen,
            "vsync" => self.settings.vsync = !self.settings.vsync,
            "colorblind_safe" => self.settings.colorblind_safe = !self.settings.colorblind_safe,
            "reduced_motion" => self.settings.reduced_motion = !self.settings.reduced_motion,
            "drag_toggle" => {
                self.settings.drag_preference = match self.settings.drag_preference {
                    crate::settings::DragPreference::Hold => {
                        crate::settings::DragPreference::Toggle
                    }
                    crate::settings::DragPreference::Toggle => {
                        crate::settings::DragPreference::Hold
                    }
                }
            }
            _ => {}
        }
        self.settings.apply();
        self.save_runtime_settings();
    }

    fn cycle_runtime_setting(&mut self, id: &str) {
        match id {
            "resolution" => {
                self.settings.resolution = match self.settings.resolution.as_str() {
                    "960x540" => "1280x720",
                    "1280x720" => "1600x900",
                    "1600x900" => "1920x1080",
                    _ => "960x540",
                }
                .to_owned();
            }
            "fps" => {
                self.settings.fps_cap = match self.settings.fps_cap {
                    30 => 60,
                    60 => 120,
                    120 => 240,
                    _ => 30,
                };
            }
            "text_size" => {
                self.settings.text_size = if self.settings.text_size >= 1.5 {
                    0.8
                } else {
                    self.settings.text_size + 0.2
                };
            }
            "master_volume" => {
                self.settings.display.master_volume =
                    cycle_volume(self.settings.display.master_volume)
            }
            "sfx_volume" => {
                self.settings.display.sfx_volume = cycle_volume(self.settings.display.sfx_volume)
            }
            "music_volume" => {
                self.settings.display.music_volume =
                    cycle_volume(self.settings.display.music_volume)
            }
            "steering" => {
                let left = self.settings.bindings.steer_left == "A";
                let _ = self
                    .settings
                    .bindings
                    .set("steer_left", if left { "Left" } else { "A" });
                let _ = self
                    .settings
                    .bindings
                    .set("steer_right", if left { "Right" } else { "D" });
            }
            "recovery" => {
                let repair = if self.settings.bindings.repair == "R" {
                    "F"
                } else {
                    "R"
                };
                let save = if self.settings.bindings.save == "S" {
                    "F5"
                } else {
                    "S"
                };
                let load = if self.settings.bindings.load == "L" {
                    "F9"
                } else {
                    "L"
                };
                let _ = self.settings.bindings.set("repair", repair);
                let _ = self.settings.bindings.set("save", save);
                let _ = self.settings.bindings.set("load", load);
            }
            "language" => {
                self.settings.language = match self.settings.language.as_str() {
                    "en" => "de",
                    "de" => "fr",
                    _ => "en",
                }
                .to_owned();
                self.localizer
                    .set_language(crate::localization::Language::from_id(
                        &self.settings.language,
                    ));
            }
            _ => {}
        }
        self.settings.sanitize();
        self.settings.apply();
        self.save_runtime_settings();
    }

    fn save_runtime_settings(&mut self) {
        if let Err(err) = self.settings.save(&self.data.config.game_name) {
            self.notifications
                .warning(format!("Preferences save failed: {err}"));
        }
    }
}

fn cycle_volume(value: f32) -> f32 {
    if value <= 0.0 {
        1.0
    } else {
        (value - 0.2).max(0.0)
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
