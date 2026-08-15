//! Generated sprites for enemies, guards, and their shots.

use super::sprites::draw_character;
use crate::state::{Enemy, EnemyKind, Guard, GuardKind, Shot};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;

fn enemy_id(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::Wolf => "wolf",
        EnemyKind::Bandit => "bandit",
        EnemyKind::BanditArcher => "bandit_archer",
        EnemyKind::Skeleton => "skeleton",
        EnemyKind::Necromancer => "necromancer",
        EnemyKind::AlphaWolf => "alpha_wolf",
        EnemyKind::ArmoredBandit => "armored_bandit",
    }
}

pub(super) fn draw_enemy(assets: &AssetManager, enemy: &Enemy) {
    let tint = if enemy.hit_flash.finished() {
        WHITE
    } else {
        Color::new(1.0, 0.72, 0.72, 1.0)
    };
    let size = if matches!(enemy.kind, EnemyKind::AlphaWolf | EnemyKind::ArmoredBandit) {
        94.0
    } else {
        82.0
    };
    draw_character(
        assets,
        enemy_id(enemy.kind),
        enemy.pos,
        vec2(size, size),
        tint,
    );
    draw_health_bar(
        vec2(enemy.pos.x - 26.0, enemy.pos.y - enemy.radius - 18.0),
        52.0,
        enemy.health,
        enemy.max_health,
        Color::new(0.78, 0.18, 0.18, 1.0),
    );
}

pub(super) fn draw_enemy_icon(assets: &AssetManager, kind: EnemyKind, pos: Vec2) {
    draw_character(assets, enemy_id(kind), pos, vec2(70.0, 70.0), WHITE);
}

pub(super) fn draw_guard(assets: &AssetManager, guard: &Guard) {
    let tint = if !guard.is_active() {
        Color::new(0.45, 0.48, 0.50, 0.62)
    } else if !guard.hit_flash.finished() {
        Color::new(0.76, 0.90, 1.0, 1.0)
    } else {
        WHITE
    };
    draw_character(assets, guard.kind.id(), guard.pos, vec2(82.0, 82.0), tint);
    draw_health_bar(
        vec2(guard.pos.x - 28.0, guard.pos.y + 30.0),
        56.0,
        guard.health,
        guard.max_health,
        Color::new(0.22, 0.68, 0.88, 1.0),
    );
}

pub(super) fn draw_guard_icon(assets: &AssetManager, kind: GuardKind, pos: Vec2, enabled: bool) {
    let tint = if enabled {
        WHITE
    } else {
        Color::new(0.55, 0.55, 0.55, 0.48)
    };
    draw_character(assets, kind.id(), pos, vec2(78.0, 78.0), tint);
}

pub(super) fn draw_shot(assets: &AssetManager, shot: &Shot) {
    let progress = 1.0 - (shot.timer / shot.total).clamp(0.0, 1.0);
    let current = shot.from + (shot.to - shot.from) * progress;
    let id = if shot.color.b > shot.color.r {
        "magic_bolt"
    } else {
        "arrow"
    };
    draw_character(assets, id, current, vec2(42.0, 42.0), WHITE);
}

fn draw_health_bar(pos: Vec2, width: f32, value: f32, max: f32, fill: Color) {
    let ratio = (value / max.max(1.0)).clamp(0.0, 1.0);
    draw_rectangle(pos.x, pos.y, width, 6.0, Color::new(0.04, 0.05, 0.05, 0.86));
    draw_rectangle(pos.x, pos.y, width * ratio, 6.0, fill);
}
