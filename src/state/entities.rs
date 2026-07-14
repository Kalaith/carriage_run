//! Mission entities and small geometry helpers.

mod enemies;
mod guards;
mod hazards;

pub use enemies::*;
pub use guards::*;
pub use hazards::*;

use macroquad::prelude::*;
use macroquad_toolkit::timing::Timer;

pub const PLAY_WIDTH: f32 = 1280.0;
pub const PLAY_TOP: f32 = 96.0;
pub const PLAY_BOTTOM: f32 = 596.0;
pub const ROAD_LEFT: f32 = 286.0;
pub const ROAD_RIGHT: f32 = 994.0;
pub const ROAD_WIDTH: f32 = ROAD_RIGHT - ROAD_LEFT;
pub const ROAD_CENTER: f32 = (ROAD_LEFT + ROAD_RIGHT) * 0.5;
pub const CARRIAGE_Y: f32 = 506.0;

pub fn road_curve_offset(y: f32, progress: f32) -> f32 {
    let primary = (progress * 0.020 + y * 0.010).sin() * 58.0;
    let secondary = (progress * 0.012 - y * 0.018).sin() * 28.0;
    (primary + secondary).clamp(-92.0, 92.0)
}

pub fn road_center_at_y(y: f32, progress: f32) -> f32 {
    ROAD_CENTER + road_curve_offset(y, progress)
}

pub fn road_left_at_y(y: f32, progress: f32) -> f32 {
    road_center_at_y(y, progress) - ROAD_WIDTH * 0.5
}

pub fn road_right_at_y(y: f32, progress: f32) -> f32 {
    road_center_at_y(y, progress) + ROAD_WIDTH * 0.5
}

pub fn clamp_x_to_road_at_y(x: f32, y: f32, progress: f32, margin: f32) -> f32 {
    x.clamp(
        road_left_at_y(y, progress) + margin,
        road_right_at_y(y, progress) - margin,
    )
}

#[derive(Debug, Clone)]
pub struct Carriage {
    pub pos: Vec2,
    pub target_x: f32,
    pub health: f32,
    pub max_health: f32,
    pub cargo: f32,
    pub max_cargo: f32,
    pub slow_timer: f32,
    pub hit_flash: Timer,
}

impl Carriage {
    pub(super) fn new(max_health: f32, max_cargo: f32) -> Self {
        Self {
            pos: vec2(ROAD_CENTER, CARRIAGE_Y),
            target_x: ROAD_CENTER,
            health: max_health,
            max_health,
            cargo: max_cargo,
            max_cargo,
            slow_timer: 0.0,
            hit_flash: Timer::new(0.0),
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x - 40.0, self.pos.y - 50.0, 80.0, 96.0)
    }

    pub(super) fn update_timers(&mut self, dt: f32) {
        self.slow_timer = (self.slow_timer - dt).max(0.0);
        self.hit_flash.tick(dt);
    }
}

#[derive(Debug, Clone)]
pub struct Shot {
    pub from: Vec2,
    pub to: Vec2,
    pub timer: f32,
    pub total: f32,
    pub color: Color,
}

impl Shot {
    pub(super) fn new(from: Vec2, to: Vec2, color: Color) -> Self {
        Self {
            from,
            to,
            timer: 0.18,
            total: 0.18,
            color,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DragState {
    None,
    Carriage,
    Guard { guard_id: u32, grab: Vec2 },
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub text: String,
    pub timer: f32,
}

impl Alert {
    pub(super) fn set(&mut self, text: &str) {
        self.text = text.to_owned();
        self.timer = 1.6;
    }

    pub(super) fn update(&mut self, dt: f32) {
        self.timer = (self.timer - dt).max(0.0);
    }
}

impl Default for Alert {
    fn default() -> Self {
        Self {
            text: String::new(),
            timer: 0.0,
        }
    }
}

pub(super) fn contains_point(rect: Rect, point: Vec2) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.w
        && point.y >= rect.y
        && point.y <= rect.y + rect.h
}

pub(super) fn clamp_to_field(point: Vec2) -> Vec2 {
    vec2(
        point.x.clamp(ROAD_LEFT - 90.0, ROAD_RIGHT + 90.0),
        point.y.clamp(PLAY_TOP + 25.0, PLAY_BOTTOM - 25.0),
    )
}

pub(super) fn move_towards(current: Vec2, target: Vec2, max_delta: f32) -> Vec2 {
    let delta = target - current;
    let dist = delta.length();
    if dist <= max_delta || dist <= f32::EPSILON {
        target
    } else {
        current + delta / dist * max_delta
    }
}
