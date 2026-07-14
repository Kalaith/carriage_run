//! Procedural sprites for enemies and guards, their shots and health bars.

use crate::state::{Enemy, EnemyKind, Guard, GuardKind, GuardOrder, Shot};
use macroquad::prelude::*;

pub(super) fn draw_enemy(enemy: &Enemy) {
    let flash = !enemy.hit_flash.finished();
    draw_circle(
        enemy.pos.x + 3.0,
        enemy.pos.y + 6.0,
        enemy.radius + 3.0,
        Color::new(0.0, 0.0, 0.0, 0.25),
    );
    draw_enemy_sprite(enemy.kind, enemy.pos, flash);
    draw_health_bar(
        vec2(enemy.pos.x - 26.0, enemy.pos.y - enemy.radius - 18.0),
        52.0,
        enemy.health,
        enemy.max_health,
        Color::new(0.78, 0.18, 0.18, 1.0),
    );
}

/// Draw an enemy's procedural sprite as a static icon (no health bar / flash),
/// reused by the field guide so players learn to recognise threats.
pub(super) fn draw_enemy_icon(kind: EnemyKind, pos: Vec2) {
    draw_circle(
        pos.x + 3.0,
        pos.y + 8.0,
        20.0,
        Color::new(0.0, 0.0, 0.0, 0.22),
    );
    draw_enemy_sprite(kind, pos, false);
}

fn draw_enemy_sprite(kind: EnemyKind, pos: Vec2, flash: bool) {
    match kind {
        EnemyKind::Wolf => draw_wolf(pos, flash),
        EnemyKind::Bandit => draw_bandit(pos, flash),
        EnemyKind::BanditArcher => draw_bandit_archer(pos, flash),
        EnemyKind::Skeleton => draw_skeleton(pos, flash),
        EnemyKind::Necromancer => draw_necromancer(pos, flash),
        EnemyKind::AlphaWolf => draw_alpha_wolf(pos, flash),
        EnemyKind::ArmoredBandit => draw_armored_bandit(pos, flash),
    }
}

/// Elite raider: a bandit clad in steel plate over the red coat, so it reads as
/// an armored bruiser.
fn draw_armored_bandit(pos: Vec2, flash: bool) {
    let coat = if flash {
        WHITE
    } else {
        Color::new(0.40, 0.10, 0.10, 1.0)
    };
    let steel = if flash {
        WHITE
    } else {
        Color::new(0.55, 0.58, 0.62, 1.0)
    };
    draw_circle(pos.x, pos.y - 11.0, 15.0, Color::new(0.74, 0.52, 0.36, 1.0));
    // Helmet.
    draw_rectangle(pos.x - 16.0, pos.y - 22.0, 32.0, 11.0, steel);
    // Coat with a steel breastplate over it.
    draw_rectangle(pos.x - 19.0, pos.y - 2.0, 38.0, 36.0, coat);
    draw_rectangle(pos.x - 14.0, pos.y + 2.0, 28.0, 26.0, steel);
    draw_rectangle(
        pos.x - 14.0,
        pos.y + 12.0,
        28.0,
        3.0,
        Color::new(0.30, 0.32, 0.34, 1.0),
    );
    // Pauldrons.
    draw_circle(pos.x - 20.0, pos.y + 2.0, 6.0, steel);
    draw_circle(pos.x + 20.0, pos.y + 2.0, 6.0, steel);
}

/// Elite wolf: a larger, darker wolf with red eyes, so it reads as a step up
/// from the common wolf at a glance.
fn draw_alpha_wolf(pos: Vec2, flash: bool) {
    let body = if flash {
        WHITE
    } else {
        Color::new(0.20, 0.20, 0.23, 1.0)
    };
    draw_circle(pos.x, pos.y, 24.0, body);
    draw_triangle(
        vec2(pos.x - 19.0, pos.y - 13.0),
        vec2(pos.x - 6.0, pos.y - 40.0),
        vec2(pos.x + 3.0, pos.y - 12.0),
        body,
    );
    draw_triangle(
        vec2(pos.x + 19.0, pos.y - 13.0),
        vec2(pos.x + 6.0, pos.y - 40.0),
        vec2(pos.x - 3.0, pos.y - 12.0),
        body,
    );
    let eye = Color::new(0.90, 0.16, 0.12, 1.0);
    draw_circle(pos.x - 8.0, pos.y - 3.0, 2.6, eye);
    draw_circle(pos.x + 8.0, pos.y - 3.0, 2.6, eye);
}

fn draw_wolf(pos: Vec2, flash: bool) {
    let body = if flash {
        WHITE
    } else {
        Color::new(0.33, 0.36, 0.38, 1.0)
    };
    draw_circle(pos.x, pos.y, 18.0, body);
    draw_triangle(
        vec2(pos.x - 14.0, pos.y - 10.0),
        vec2(pos.x - 4.0, pos.y - 30.0),
        vec2(pos.x + 2.0, pos.y - 9.0),
        body,
    );
    draw_triangle(
        vec2(pos.x + 14.0, pos.y - 10.0),
        vec2(pos.x + 4.0, pos.y - 30.0),
        vec2(pos.x - 2.0, pos.y - 9.0),
        body,
    );
    draw_circle(pos.x - 6.0, pos.y - 2.0, 2.0, BLACK);
    draw_circle(pos.x + 6.0, pos.y - 2.0, 2.0, BLACK);
}

fn draw_bandit(pos: Vec2, flash: bool) {
    let coat = if flash {
        WHITE
    } else {
        Color::new(0.48, 0.10, 0.10, 1.0)
    };
    draw_circle(pos.x, pos.y - 10.0, 14.0, Color::new(0.78, 0.55, 0.38, 1.0));
    draw_rectangle(pos.x - 17.0, pos.y - 2.0, 34.0, 34.0, coat);
    draw_rectangle(
        pos.x - 18.0,
        pos.y - 18.0,
        36.0,
        10.0,
        Color::new(0.18, 0.05, 0.05, 1.0),
    );
    draw_circle(
        pos.x + 22.0,
        pos.y + 8.0,
        9.0,
        Color::new(0.74, 0.54, 0.24, 1.0),
    );
}

fn draw_bandit_archer(pos: Vec2, flash: bool) {
    let coat = if flash {
        WHITE
    } else {
        Color::new(0.38, 0.16, 0.10, 1.0)
    };
    draw_circle(pos.x, pos.y - 10.0, 13.0, Color::new(0.78, 0.55, 0.38, 1.0));
    draw_rectangle(pos.x - 15.0, pos.y - 1.0, 30.0, 32.0, coat);
    draw_line(
        pos.x - 23.0,
        pos.y + 2.0,
        pos.x + 24.0,
        pos.y - 16.0,
        3.0,
        Color::new(0.80, 0.62, 0.36, 1.0),
    );
    draw_circle_lines(
        pos.x + 20.0,
        pos.y - 8.0,
        16.0,
        2.0,
        Color::new(0.80, 0.62, 0.36, 1.0),
    );
}

fn draw_skeleton(pos: Vec2, flash: bool) {
    let bone = if flash {
        WHITE
    } else {
        Color::new(0.82, 0.84, 0.78, 1.0)
    };
    draw_circle(pos.x, pos.y - 14.0, 13.0, bone);
    draw_rectangle(pos.x - 10.0, pos.y, 20.0, 28.0, bone);
    draw_line(
        pos.x - 18.0,
        pos.y + 5.0,
        pos.x + 18.0,
        pos.y + 5.0,
        5.0,
        bone,
    );
    draw_line(
        pos.x - 8.0,
        pos.y + 26.0,
        pos.x - 18.0,
        pos.y + 42.0,
        5.0,
        bone,
    );
    draw_line(
        pos.x + 8.0,
        pos.y + 26.0,
        pos.x + 18.0,
        pos.y + 42.0,
        5.0,
        bone,
    );
    draw_circle(pos.x - 5.0, pos.y - 16.0, 2.0, BLACK);
    draw_circle(pos.x + 5.0, pos.y - 16.0, 2.0, BLACK);
}

fn draw_necromancer(pos: Vec2, flash: bool) {
    let robe = if flash {
        WHITE
    } else {
        Color::new(0.22, 0.13, 0.34, 1.0)
    };
    draw_circle(pos.x, pos.y - 13.0, 14.0, Color::new(0.70, 0.76, 0.66, 1.0));
    draw_triangle(
        vec2(pos.x, pos.y - 2.0),
        vec2(pos.x - 24.0, pos.y + 38.0),
        vec2(pos.x + 24.0, pos.y + 38.0),
        robe,
    );
    draw_line(
        pos.x + 20.0,
        pos.y + 24.0,
        pos.x + 32.0,
        pos.y - 28.0,
        4.0,
        Color::new(0.44, 0.25, 0.12, 1.0),
    );
    draw_circle(
        pos.x + 34.0,
        pos.y - 32.0,
        7.0,
        Color::new(0.46, 0.86, 0.72, 1.0),
    );
}

pub(super) fn draw_guard(guard: &Guard) {
    let down = !guard.is_active();
    let base_color = match guard.kind {
        GuardKind::Swordsman => Color::new(0.18, 0.42, 0.64, 1.0),
        GuardKind::ShieldGuard => Color::new(0.20, 0.46, 0.36, 1.0),
        GuardKind::Spearman => Color::new(0.42, 0.34, 0.64, 1.0),
        GuardKind::Archer => Color::new(0.18, 0.46, 0.28, 1.0),
        GuardKind::CrossbowGuard => Color::new(0.42, 0.38, 0.32, 1.0),
        GuardKind::Mage => Color::new(0.26, 0.34, 0.68, 1.0),
    };
    let body = if down {
        Color::new(0.20, 0.23, 0.25, 0.72)
    } else if !guard.hit_flash.finished() {
        WHITE
    } else {
        base_color
    };
    draw_circle(
        guard.pos.x + 3.0,
        guard.pos.y + 6.0,
        23.0,
        Color::new(0.0, 0.0, 0.0, 0.25),
    );
    draw_circle(guard.pos.x, guard.pos.y, 21.0, body);
    draw_rectangle(
        guard.pos.x - 6.0,
        guard.pos.y - 27.0,
        12.0,
        18.0,
        Color::new(0.76, 0.66, 0.45, 1.0),
    );
    draw_guard_weapon(guard);
    if !down && guard.mounted_slot.is_none() {
        // Stance ring: read a guard's standing order at a glance.
        let stance_ring = match guard.order {
            GuardOrder::Attack(_) => Some(Color::new(0.95, 0.76, 0.28, 0.72)),
            GuardOrder::Roam => Some(Color::new(0.86, 0.44, 0.24, 0.60)),
            GuardOrder::Hold | GuardOrder::Move(_) => Some(Color::new(0.42, 0.70, 0.95, 0.55)),
            GuardOrder::Escort => None,
        };
        if let Some(color) = stance_ring {
            draw_circle_lines(guard.pos.x, guard.pos.y, 31.0, 2.0, color);
        }
    }
    if guard.attack_flash > 0.0 {
        let alpha = (guard.attack_flash / 0.16).clamp(0.0, 1.0);
        draw_circle_lines(
            guard.pos.x,
            guard.pos.y,
            if guard.kind.is_ranged() { 31.0 } else { 38.0 },
            3.0,
            Color::new(1.0, 0.84, 0.34, alpha),
        );
    }
    draw_health_bar(
        vec2(guard.pos.x - 28.0, guard.pos.y + 30.0),
        56.0,
        guard.health,
        guard.max_health,
        Color::new(0.22, 0.68, 0.88, 1.0),
    );
}

fn draw_guard_weapon(guard: &Guard) {
    match guard.kind {
        GuardKind::Swordsman => draw_line(
            guard.pos.x + 15.0,
            guard.pos.y - 4.0,
            guard.pos.x + 34.0,
            guard.pos.y - 22.0,
            4.0,
            Color::new(0.86, 0.88, 0.82, 1.0),
        ),
        GuardKind::ShieldGuard => {
            draw_circle(
                guard.pos.x + 20.0,
                guard.pos.y + 2.0,
                13.0,
                Color::new(0.64, 0.70, 0.64, 1.0),
            );
            draw_circle_lines(
                guard.pos.x + 20.0,
                guard.pos.y + 2.0,
                13.0,
                2.0,
                Color::new(0.22, 0.26, 0.22, 1.0),
            );
        }
        GuardKind::Spearman => draw_line(
            guard.pos.x - 18.0,
            guard.pos.y + 12.0,
            guard.pos.x + 38.0,
            guard.pos.y - 30.0,
            4.0,
            Color::new(0.82, 0.72, 0.45, 1.0),
        ),
        GuardKind::Archer => draw_line(
            guard.pos.x - 13.0,
            guard.pos.y - 2.0,
            guard.pos.x + 27.0,
            guard.pos.y - 18.0,
            3.0,
            Color::new(0.95, 0.80, 0.38, 1.0),
        ),
        GuardKind::CrossbowGuard => {
            draw_rectangle(
                guard.pos.x + 9.0,
                guard.pos.y - 12.0,
                28.0,
                8.0,
                Color::new(0.75, 0.70, 0.58, 1.0),
            );
            draw_line(
                guard.pos.x + 14.0,
                guard.pos.y - 22.0,
                guard.pos.x + 33.0,
                guard.pos.y + 5.0,
                2.0,
                Color::new(0.86, 0.82, 0.70, 1.0),
            );
        }
        GuardKind::Mage => {
            draw_circle(
                guard.pos.x + 22.0,
                guard.pos.y - 20.0,
                7.0,
                Color::new(0.58, 0.86, 1.0, 1.0),
            );
            draw_line(
                guard.pos.x + 14.0,
                guard.pos.y - 4.0,
                guard.pos.x + 26.0,
                guard.pos.y - 28.0,
                3.0,
                Color::new(0.68, 0.52, 0.30, 1.0),
            );
        }
    }
}

pub(super) fn draw_shot(shot: &Shot) {
    let progress = 1.0 - (shot.timer / shot.total).clamp(0.0, 1.0);
    let current = shot.from + (shot.to - shot.from) * progress;
    let dir = (shot.to - shot.from).normalize_or_zero();
    draw_line(
        current.x - dir.x * 20.0,
        current.y - dir.y * 20.0,
        current.x + dir.x * 8.0,
        current.y + dir.y * 8.0,
        3.0,
        shot.color,
    );
}

fn draw_health_bar(pos: Vec2, width: f32, value: f32, max: f32, fill: Color) {
    let ratio = (value / max.max(1.0)).clamp(0.0, 1.0);
    draw_rectangle(pos.x, pos.y, width, 6.0, Color::new(0.04, 0.05, 0.05, 0.86));
    draw_rectangle(pos.x, pos.y, width * ratio, 6.0, fill);
}
