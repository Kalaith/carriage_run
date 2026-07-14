//! Active mission rendering.

use super::carriage;
use super::gameplay_hud::draw_gameplay_hud;
use super::{UiAction, UiContext, LOGICAL_WIDTH};
use crate::state::{
    road_center_at_y, road_left_at_y, road_right_at_y, DragState, Enemy, EnemyKind, Guard,
    GuardKind, GuardOrder, Hazard, HazardKind, MissionRun, PLAY_BOTTOM, PLAY_TOP, ROAD_WIDTH,
};
use macroquad::prelude::*;

pub(super) fn draw_gameplay(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let Some(run) = &ctx.session.mission else {
        return;
    };

    draw_road(run, ctx.session.campaign.route_motion_enabled);
    for hazard in &run.hazards {
        draw_hazard(hazard);
    }
    for shot in &run.shots {
        draw_shot(shot);
    }
    for enemy in &run.enemies {
        draw_enemy(enemy);
    }
    for guard in run
        .guards
        .iter()
        .filter(|guard| guard.mounted_slot.is_none())
    {
        draw_guard(guard);
    }
    if ctx.session.campaign.route_motion_enabled {
        draw_wheel_dust(run);
    }
    carriage::draw_carriage(run);
    for guard in run
        .guards
        .iter()
        .filter(|guard| guard.mounted_slot.is_some())
    {
        draw_guard(guard);
    }
    draw_drag_feedback(run, mouse);
    draw_gameplay_hud(ctx, run, mouse, actions);
}

fn draw_road(run: &MissionRun, route_motion_enabled: bool) {
    let road_scroll = if route_motion_enabled {
        run.road_scroll
    } else {
        0.0
    };
    // Continuous scroll for roadside terrain. Unlike `road_scroll` (which wraps
    // every 96px and made parallax layers stutter), this advances smoothly so
    // props track the ground and sell the sense of travel.
    let terrain = if route_motion_enabled {
        run.terrain_scroll
    } else {
        0.0
    };

    draw_rectangle(
        0.0,
        PLAY_TOP,
        LOGICAL_WIDTH,
        PLAY_BOTTOM - PLAY_TOP,
        Color::new(0.07, 0.24, 0.13, 1.0),
    );
    draw_rectangle(
        0.0,
        PLAY_TOP,
        LOGICAL_WIDTH,
        PLAY_BOTTOM - PLAY_TOP,
        Color::new(0.05, 0.12, 0.08, 0.18),
    );
    let progress = run.progress;
    draw_winding_road_band(
        progress,
        ROAD_WIDTH * 0.5 + 54.0,
        Color::new(0.16, 0.12, 0.08, 1.0),
    );
    draw_winding_road_band(
        progress,
        ROAD_WIDTH * 0.5,
        Color::new(0.43, 0.32, 0.19, 1.0),
    );
    draw_winding_road_band(
        progress,
        ROAD_WIDTH * 0.5 - 10.0,
        Color::new(0.54, 0.41, 0.24, 0.28),
    );
    draw_winding_road_edges(
        progress,
        ROAD_WIDTH * 0.5,
        4.0,
        Color::new(0.30, 0.24, 0.16, 1.0),
    );
    draw_winding_road_edges(
        progress,
        ROAD_WIDTH * 0.5 + 36.0,
        12.0,
        Color::new(0.08, 0.08, 0.05, 0.28),
    );

    for i in -1..8 {
        let y = PLAY_TOP + i as f32 * 104.0 + road_scroll;
        let lane_x = road_center_at_y(y, progress);
        draw_rectangle(
            lane_x - 4.0,
            y,
            8.0,
            48.0,
            Color::new(0.76, 0.66, 0.42, 0.62),
        );
    }

    let play_h = PLAY_BOTTOM - PLAY_TOP;

    // Distant parallax layer: faint silhouettes near the screen edges that drift
    // slower than the ground for a sense of depth.
    for i in 0..12 {
        let y = PLAY_TOP + ((i as f32 * 88.0 + terrain * 0.45).rem_euclid(play_h));
        let left_x = 18.0 + (i % 3) as f32 * 26.0;
        let right_x = 1200.0 - (i % 4) as f32 * 24.0;
        draw_far_silhouette(vec2(left_x, y), 0.62 + (i % 3) as f32 * 0.08);
        draw_far_silhouette(vec2(right_x, y + 34.0), 0.58 + (i % 2) as f32 * 0.1);
    }

    for i in 0..22 {
        let y = PLAY_TOP + ((i as f32 * 67.0 + terrain).rem_euclid(play_h));
        let left_x = 48.0 + (i % 5) as f32 * 38.0;
        let right_x = 1058.0 + (i % 6) as f32 * 28.0;
        if i % 3 == 0 {
            draw_tree_cluster(vec2(left_x + 22.0, y), 0.88 + (i % 4) as f32 * 0.09);
            draw_tree_cluster(vec2(right_x + 8.0, y + 26.0), 0.82 + (i % 3) as f32 * 0.12);
        } else {
            draw_bush(vec2(left_x, y));
            draw_bush(vec2(right_x, y + 18.0));
        }
        draw_grass_tuft(vec2(left_x + 70.0, y + 30.0), 0.7);
        draw_grass_tuft(vec2(right_x - 46.0, y + 8.0), 0.62);
    }

    for i in 0..26 {
        let y = PLAY_TOP + ((i as f32 * 49.0 + terrain).rem_euclid(play_h));
        let left = road_left_at_y(y, progress) + 54.0;
        let right = road_right_at_y(y, progress) - 54.0;
        let x = left + (((i * 83) % 590) as f32 / 590.0) * (right - left);
        draw_road_pebble(vec2(x, y), 0.7 + (i % 3) as f32 * 0.22);
    }

    let finish_alpha = (run.progress_ratio() - 0.86).max(0.0) / 0.14;
    if finish_alpha > 0.0 {
        let y = PLAY_TOP + 22.0;
        draw_line(
            road_left_at_y(y, progress),
            y,
            road_right_at_y(y, progress),
            y,
            16.0,
            Color::new(0.92, 0.86, 0.55, finish_alpha.min(1.0)),
        );
    }
}

fn draw_winding_road_band(progress: f32, half_width: f32, color: Color) {
    let mut y = PLAY_TOP;
    while y < PLAY_BOTTOM {
        let next_y = (y + 20.0).min(PLAY_BOTTOM);
        let top = road_center_at_y(y, progress);
        let bottom = road_center_at_y(next_y, progress);
        draw_road_segment(top, bottom, y, next_y, half_width, color);
        y = next_y;
    }
}

fn draw_road_segment(top: f32, bottom: f32, y: f32, next_y: f32, half_width: f32, color: Color) {
    let tl = vec2(top - half_width, y);
    let tr = vec2(top + half_width, y);
    let bl = vec2(bottom - half_width, next_y);
    let br = vec2(bottom + half_width, next_y);
    draw_triangle(tl, tr, bl, color);
    draw_triangle(tr, br, bl, color);
}

fn draw_winding_road_edges(progress: f32, half_width: f32, thickness: f32, color: Color) {
    let mut y = PLAY_TOP;
    while y < PLAY_BOTTOM {
        let next_y = (y + 18.0).min(PLAY_BOTTOM);
        let top = road_center_at_y(y, progress);
        let bottom = road_center_at_y(next_y, progress);
        draw_line(
            top - half_width,
            y,
            bottom - half_width,
            next_y,
            thickness,
            color,
        );
        draw_line(
            top + half_width,
            y,
            bottom + half_width,
            next_y,
            thickness,
            color,
        );
        y = next_y;
    }
}

fn draw_bush(pos: Vec2) {
    draw_circle(pos.x, pos.y, 13.0, Color::new(0.08, 0.25, 0.13, 1.0));
    draw_circle(
        pos.x + 12.0,
        pos.y + 3.0,
        10.0,
        Color::new(0.10, 0.31, 0.17, 1.0),
    );
    draw_circle(
        pos.x - 8.0,
        pos.y + 6.0,
        8.0,
        Color::new(0.07, 0.22, 0.12, 1.0),
    );
}

fn draw_tree_cluster(pos: Vec2, scale: f32) {
    draw_circle(
        pos.x + 8.0 * scale,
        pos.y + 16.0 * scale,
        9.0 * scale,
        Color::new(0.06, 0.07, 0.04, 0.34),
    );
    draw_rectangle(
        pos.x - 4.0 * scale,
        pos.y + 10.0 * scale,
        8.0 * scale,
        18.0 * scale,
        Color::new(0.18, 0.10, 0.045, 1.0),
    );
    for (dx, dy, r, color) in [
        (-14.0, -6.0, 18.0, Color::new(0.10, 0.30, 0.14, 1.0)),
        (2.0, -14.0, 22.0, Color::new(0.13, 0.36, 0.15, 1.0)),
        (16.0, -2.0, 17.0, Color::new(0.08, 0.25, 0.12, 1.0)),
        (-2.0, 6.0, 20.0, Color::new(0.12, 0.33, 0.15, 1.0)),
    ] {
        draw_circle(pos.x + dx * scale, pos.y + dy * scale, r * scale, color);
    }
    draw_circle(
        pos.x - 6.0 * scale,
        pos.y - 13.0 * scale,
        8.0 * scale,
        Color::new(0.22, 0.44, 0.16, 0.70),
    );
}

fn draw_far_silhouette(pos: Vec2, scale: f32) {
    let color = Color::new(0.05, 0.14, 0.09, 0.55);
    draw_rectangle(
        pos.x - 2.0 * scale,
        pos.y + 6.0 * scale,
        4.0 * scale,
        12.0 * scale,
        Color::new(0.09, 0.08, 0.05, 0.5),
    );
    draw_circle(pos.x, pos.y, 12.0 * scale, color);
    draw_circle(pos.x - 7.0 * scale, pos.y + 4.0 * scale, 8.0 * scale, color);
    draw_circle(pos.x + 7.0 * scale, pos.y + 3.0 * scale, 9.0 * scale, color);
}

fn draw_grass_tuft(pos: Vec2, scale: f32) {
    let color = Color::new(0.18, 0.43, 0.18, 0.75);
    for offset in [-10.0_f32, -5.0, 0.0, 5.0, 10.0] {
        draw_line(
            pos.x,
            pos.y,
            pos.x + offset * scale,
            pos.y - (15.0 - offset.abs() * 0.3) * scale,
            2.0,
            color,
        );
    }
}

fn draw_road_pebble(pos: Vec2, scale: f32) {
    let color = Color::new(0.36, 0.29, 0.18, 0.62);
    draw_circle(pos.x, pos.y, 3.0 * scale, color);
    draw_circle(
        pos.x + 11.0 * scale,
        pos.y + 5.0 * scale,
        2.2 * scale,
        Color::new(0.31, 0.24, 0.15, 0.50),
    );
}

/// Draw a hazard's procedural look as a fixed-size icon for the field guide,
/// independent of a live `Hazard` instance.
pub(super) fn draw_hazard_icon(kind: HazardKind, pos: Vec2) {
    match kind {
        HazardKind::Mud => {
            let color = Color::new(0.16, 0.10, 0.07, 0.95);
            draw_circle(pos.x, pos.y, 18.0, color);
            draw_circle(pos.x + 13.0, pos.y - 4.0, 11.0, color);
            draw_circle(pos.x - 11.0, pos.y + 5.0, 10.0, color);
            draw_circle_lines(pos.x, pos.y, 18.0, 2.0, Color::new(0.42, 0.29, 0.17, 0.8));
        }
        HazardKind::FallenTree => {
            let trunk = Color::new(0.38, 0.21, 0.11, 1.0);
            draw_rectangle(pos.x - 24.0, pos.y - 7.0, 48.0, 14.0, trunk);
            draw_rectangle_lines(
                pos.x - 24.0,
                pos.y - 7.0,
                48.0,
                14.0,
                2.0,
                Color::new(0.16, 0.08, 0.04, 1.0),
            );
            draw_line(
                pos.x - 12.0,
                pos.y - 7.0,
                pos.x - 20.0,
                pos.y - 22.0,
                6.0,
                trunk,
            );
            draw_line(
                pos.x + 14.0,
                pos.y + 7.0,
                pos.x + 22.0,
                pos.y + 20.0,
                5.0,
                trunk,
            );
        }
        HazardKind::Rocks => {
            let color = Color::new(0.36, 0.35, 0.32, 1.0);
            draw_circle(pos.x - 12.0, pos.y + 6.0, 13.0, color);
            draw_circle(
                pos.x + 8.0,
                pos.y - 4.0,
                16.0,
                Color::new(0.41, 0.40, 0.37, 1.0),
            );
            draw_circle(pos.x + 18.0, pos.y + 9.0, 10.0, color);
        }
        HazardKind::FirePatch => {
            draw_circle(pos.x, pos.y, 18.0, Color::new(0.90, 0.24, 0.08, 0.95));
            draw_circle(
                pos.x - 8.0,
                pos.y + 5.0,
                11.0,
                Color::new(1.0, 0.62, 0.14, 0.95),
            );
            draw_circle(
                pos.x + 9.0,
                pos.y - 4.0,
                8.0,
                Color::new(0.96, 0.80, 0.20, 0.95),
            );
        }
        HazardKind::RiverFord => {
            draw_rectangle(
                pos.x - 22.0,
                pos.y - 14.0,
                44.0,
                28.0,
                Color::new(0.16, 0.34, 0.46, 0.92),
            );
            for i in 0..2 {
                let y = pos.y - 5.0 + i as f32 * 10.0;
                draw_line(
                    pos.x - 17.0,
                    y,
                    pos.x + 17.0,
                    y,
                    2.0,
                    Color::new(0.62, 0.80, 0.88, 0.7),
                );
            }
        }
    }
}

fn draw_hazard(hazard: &Hazard) {
    match hazard.kind {
        HazardKind::Mud => {
            let color = if hazard.triggered {
                Color::new(0.20, 0.14, 0.09, 0.82)
            } else {
                Color::new(0.16, 0.10, 0.07, 0.92)
            };
            draw_circle(hazard.pos.x, hazard.pos.y, hazard.radius, color);
            draw_circle(
                hazard.pos.x + 24.0,
                hazard.pos.y - 5.0,
                hazard.radius * 0.62,
                color,
            );
            draw_circle(
                hazard.pos.x - 20.0,
                hazard.pos.y + 8.0,
                hazard.radius * 0.56,
                color,
            );
            draw_circle_lines(
                hazard.pos.x,
                hazard.pos.y,
                hazard.radius,
                2.0,
                Color::new(0.42, 0.29, 0.17, 0.7),
            );
        }
        HazardKind::FallenTree => {
            let rect = hazard.rect();
            let trunk = if hazard.active {
                Color::new(0.38, 0.21, 0.11, 1.0)
            } else {
                Color::new(0.25, 0.16, 0.10, 0.65)
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, trunk);
            draw_rectangle_lines(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                2.0,
                Color::new(0.16, 0.08, 0.04, 1.0),
            );
            draw_line(
                rect.x + 42.0,
                rect.y,
                rect.x + 10.0,
                rect.y - 25.0,
                8.0,
                trunk,
            );
            draw_line(
                rect.right() - 52.0,
                rect.bottom(),
                rect.right() - 14.0,
                rect.bottom() + 23.0,
                7.0,
                trunk,
            );
        }
        HazardKind::Rocks => {
            let color = if hazard.active {
                Color::new(0.36, 0.35, 0.32, 1.0)
            } else {
                Color::new(0.22, 0.21, 0.20, 0.55)
            };
            draw_circle(hazard.pos.x - 14.0, hazard.pos.y + 8.0, 17.0, color);
            draw_circle(
                hazard.pos.x + 10.0,
                hazard.pos.y - 5.0,
                21.0,
                Color::new(color.r + 0.05, color.g + 0.05, color.b + 0.05, color.a),
            );
            draw_circle(hazard.pos.x + 24.0, hazard.pos.y + 12.0, 13.0, color);
        }
        HazardKind::FirePatch => {
            let alpha = if hazard.triggered { 0.72 } else { 0.92 };
            draw_circle(
                hazard.pos.x,
                hazard.pos.y,
                hazard.radius,
                Color::new(0.90, 0.24, 0.08, alpha),
            );
            draw_circle(
                hazard.pos.x - 15.0,
                hazard.pos.y + 9.0,
                hazard.radius * 0.58,
                Color::new(1.0, 0.62, 0.14, alpha),
            );
            draw_circle(
                hazard.pos.x + 20.0,
                hazard.pos.y - 7.0,
                hazard.radius * 0.46,
                Color::new(0.96, 0.80, 0.20, alpha),
            );
        }
        HazardKind::RiverFord => {
            let rect = hazard.rect();
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.16, 0.34, 0.46, 0.72),
            );
            // Ripple lines across the ford.
            for i in 0..3 {
                let y = rect.y + rect.h * (0.28 + i as f32 * 0.22);
                draw_line(
                    rect.x + 8.0,
                    y,
                    rect.right() - 8.0,
                    y,
                    2.0,
                    Color::new(0.62, 0.80, 0.88, 0.55),
                );
            }
        }
    }
}

fn draw_enemy(enemy: &Enemy) {
    let flash = enemy.hit_flash > 0.0;
    draw_circle(
        enemy.pos.x + 3.0,
        enemy.pos.y + 6.0,
        enemy.radius + 3.0,
        Color::new(0.0, 0.0, 0.0, 0.25),
    );
    match enemy.kind {
        EnemyKind::Wolf => draw_wolf(enemy.pos, flash),
        EnemyKind::Bandit => draw_bandit(enemy.pos, flash),
        EnemyKind::BanditArcher => draw_bandit_archer(enemy.pos, flash),
        EnemyKind::Skeleton => draw_skeleton(enemy.pos, flash),
        EnemyKind::Necromancer => draw_necromancer(enemy.pos, flash),
        EnemyKind::AlphaWolf => draw_alpha_wolf(enemy.pos, flash),
        EnemyKind::ArmoredBandit => draw_armored_bandit(enemy.pos, flash),
    }
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
    match kind {
        EnemyKind::Wolf => draw_wolf(pos, false),
        EnemyKind::Bandit => draw_bandit(pos, false),
        EnemyKind::BanditArcher => draw_bandit_archer(pos, false),
        EnemyKind::Skeleton => draw_skeleton(pos, false),
        EnemyKind::Necromancer => draw_necromancer(pos, false),
        EnemyKind::AlphaWolf => draw_alpha_wolf(pos, false),
        EnemyKind::ArmoredBandit => draw_armored_bandit(pos, false),
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

fn draw_guard(guard: &Guard) {
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
    } else if guard.hit_flash > 0.0 {
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

fn draw_shot(shot: &crate::state::Shot) {
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

fn draw_drag_feedback(run: &MissionRun, mouse: Vec2) {
    match run.drag {
        DragState::Carriage => {
            draw_line(
                run.carriage.pos.x,
                run.carriage.pos.y,
                mouse.x,
                run.carriage.pos.y - 62.0,
                3.0,
                Color::new(0.95, 0.78, 0.28, 0.85),
            );
            draw_triangle(
                vec2(mouse.x, run.carriage.pos.y - 82.0),
                vec2(mouse.x - 12.0, run.carriage.pos.y - 60.0),
                vec2(mouse.x + 12.0, run.carriage.pos.y - 60.0),
                Color::new(0.95, 0.78, 0.28, 0.85),
            );
        }
        DragState::Guard { guard_id, .. } => {
            if let Some(guard) = run.guards.iter().find(|guard| guard.id == guard_id) {
                draw_line(
                    guard.pos.x,
                    guard.pos.y,
                    mouse.x,
                    mouse.y,
                    3.0,
                    Color::new(0.38, 0.78, 0.98, 0.88),
                );
                draw_circle_lines(
                    mouse.x,
                    mouse.y,
                    26.0,
                    2.0,
                    Color::new(0.38, 0.78, 0.98, 0.88),
                );
                if guard.kind.is_ranged() {
                    for slot in 0..run.ranged_slots {
                        let pos = run.carriage_slot_pos(slot);
                        draw_circle_lines(
                            pos.x,
                            pos.y,
                            24.0,
                            3.0,
                            Color::new(0.95, 0.78, 0.28, 0.88),
                        );
                    }
                }
            }
        }
        DragState::None => {}
    }
}

fn draw_wheel_dust(run: &MissionRun) {
    let intensity = run.speed_ratio();
    if intensity <= 0.05 {
        return;
    }
    let rect = run.carriage.rect();
    let scroll = run.terrain_scroll;
    let tint = if run.is_slowed() {
        Color::new(0.34, 0.24, 0.14, 1.0)
    } else {
        Color::new(0.60, 0.52, 0.38, 1.0)
    };
    for &wheel_x in &[rect.x + 12.0, rect.right() - 12.0] {
        for i in 0..4 {
            let phase = (scroll * 0.9 + i as f32 * 15.0).rem_euclid(46.0) / 46.0;
            let y = rect.bottom() - 8.0 + phase * 40.0;
            let alpha = (1.0 - phase) * 0.30 * intensity;
            let radius = 3.5 + phase * 7.0;
            draw_circle(
                wheel_x + (i as f32 - 1.5) * 3.5,
                y,
                radius,
                Color::new(tint.r, tint.g, tint.b, alpha),
            );
        }
    }
}

fn draw_health_bar(pos: Vec2, width: f32, value: f32, max: f32, fill: Color) {
    let ratio = (value / max.max(1.0)).clamp(0.0, 1.0);
    draw_rectangle(pos.x, pos.y, width, 6.0, Color::new(0.04, 0.05, 0.05, 0.86));
    draw_rectangle(pos.x, pos.y, width * ratio, 6.0, fill);
}
