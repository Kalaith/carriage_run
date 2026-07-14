//! The winding road, its roadside terrain, and wheel dust.

use super::LOGICAL_WIDTH;
use crate::state::{
    road_center_at_y, road_left_at_y, road_right_at_y, MissionRun, PLAY_BOTTOM, PLAY_TOP,
    ROAD_WIDTH,
};
use macroquad::prelude::*;

pub(super) fn draw_road(run: &MissionRun, route_motion_enabled: bool) {
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

pub(super) fn draw_wheel_dust(run: &MissionRun) {
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
