//! Player-facing display, accessibility, audio, and rebinding preferences.

use macroquad::prelude::KeyCode;
use macroquad_toolkit::settings::GameSettings;
use serde::{Deserialize, Serialize};

pub const SETTINGS_KEY: &str = "carriage_run_settings";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DragPreference {
    #[default]
    Hold,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBindings {
    pub steer_left: String,
    pub steer_right: String,
    pub boost: String,
    pub brake: String,
    pub repair: String,
    pub save: String,
    pub load: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            steer_left: "A".to_owned(),
            steer_right: "D".to_owned(),
            boost: "Space".to_owned(),
            brake: "LeftShift".to_owned(),
            repair: "R".to_owned(),
            save: "S".to_owned(),
            load: "L".to_owned(),
        }
    }
}

impl KeyBindings {
    pub fn key(&self, action: &str) -> KeyCode {
        let value = match action {
            "steer_left" => &self.steer_left,
            "steer_right" => &self.steer_right,
            "boost" => &self.boost,
            "brake" => &self.brake,
            "repair" => &self.repair,
            "save" => &self.save,
            "load" => &self.load,
            _ => return KeyCode::Unknown,
        };
        parse_key(value)
    }

    pub fn set(&mut self, action: &str, key: &str) -> bool {
        let target = match action {
            "steer_left" => &mut self.steer_left,
            "steer_right" => &mut self.steer_right,
            "boost" => &mut self.boost,
            "brake" => &mut self.brake,
            "repair" => &mut self.repair,
            "save" => &mut self.save,
            "load" => &mut self.load,
            _ => return false,
        };
        *target = key.to_owned();
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeSettings {
    pub display: GameSettings,
    pub resolution: String,
    pub vsync: bool,
    pub fps_cap: u32,
    pub colorblind_safe: bool,
    pub text_size: f32,
    pub reduced_motion: bool,
    pub drag_preference: DragPreference,
    pub bindings: KeyBindings,
    pub language: String,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            display: GameSettings::default(),
            resolution: "1280x720".to_owned(),
            vsync: true,
            fps_cap: 60,
            colorblind_safe: false,
            text_size: 1.0,
            reduced_motion: false,
            drag_preference: DragPreference::Hold,
            bindings: KeyBindings::default(),
            language: "en".to_owned(),
        }
    }
}

impl RuntimeSettings {
    pub fn load(game_name: &str) -> Self {
        let mut settings: Self =
            macroquad_toolkit::persistence::load_json_key(game_name, SETTINGS_KEY)
                .unwrap_or_default();
        settings.sanitize();
        settings
    }

    pub fn save(&self, game_name: &str) -> Result<(), String> {
        macroquad_toolkit::persistence::save_json_key(game_name, SETTINGS_KEY, self)
    }

    pub fn sanitize(&mut self) {
        self.display.sanitize();
        self.resolution = match self.resolution.as_str() {
            "960x540" | "1280x720" | "1600x900" | "1920x1080" => self.resolution.clone(),
            _ => "1280x720".to_owned(),
        };
        self.fps_cap = self.fps_cap.clamp(30, 240);
        self.text_size = self.text_size.clamp(0.8, 2.0);
        if self.language != "de" && self.language != "fr" {
            self.language = "en".to_owned();
        }
    }

    pub fn apply(&self) {
        let mut display = self.display.clone();
        display.reduced_motion = self.reduced_motion;
        display.apply_display();
        macroquad_toolkit::ui::set_ui_text_scale(self.text_size * self.display.ui_text_scale);
    }

    pub fn audio_visible(&self, page_focused: bool) -> bool {
        page_focused && self.display.master_volume > 0.0
    }
}

pub fn parse_key(key: &str) -> KeyCode {
    match key {
        "A" => KeyCode::A,
        "D" => KeyCode::D,
        "Space" => KeyCode::Space,
        "LeftShift" => KeyCode::LeftShift,
        "R" => KeyCode::R,
        "S" => KeyCode::S,
        "L" => KeyCode::L,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "F" => KeyCode::F,
        "F5" => KeyCode::F5,
        "F9" => KeyCode::F9,
        _ => KeyCode::Unknown,
    }
}

pub fn colorblind_palette(enabled: bool) -> [(f32, f32, f32); 4] {
    if enabled {
        [
            (0.0, 0.45, 0.70),
            (0.90, 0.60, 0.0),
            (0.80, 0.15, 0.35),
            (0.35, 0.70, 0.20),
        ]
    } else {
        [
            (0.78, 0.12, 0.10),
            (0.88, 0.60, 0.08),
            (0.24, 0.68, 0.30),
            (0.44, 0.56, 0.86),
        ]
    }
}

#[cfg(test)]
mod tests;
