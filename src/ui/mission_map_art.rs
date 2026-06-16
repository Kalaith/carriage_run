//! Thumbnail, badge, and small icon drawing for the routes screen.

use super::upgrade_visuals::{draw_panel_with_fill, GOLD as UI_GOLD, GOLD_SOFT, INK, MUTED};
use crate::data::MissionDef;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_mission_thumbnail(rect: Rect, mission: &MissionDef) {
    draw_panel_with_fill(rect, thumbnail_background(&mission.mission_type), false);
    let inner = rect.inset(7.0);
    match mission.mission_type.as_str() {
        "time_delivery" => draw_hourglass(inner),
        "prisoner_escort" => draw_prison_wagon(inner),
        "medicine_run" => draw_medicine_bottle(inner),
        "gold_shipment" => draw_gold_stacks(inner),
        "monster_egg_transport" => draw_monster_egg(inner),
        "refugee_escort" => draw_refugee_wagon(inner),
        "princess_escort" => draw_crown(inner),
        "royal_banquet_supplies" => draw_banquet(inner),
        "siege_supply_run" => draw_catapult(inner),
        _ => draw_forest_track(inner, mission.order),
    }
}

pub(super) fn draw_type_badge(rect: Rect, mission_type: &str) {
    let (label, color) = mission_type_style(mission_type);
    draw_panel_with_fill(rect, color, false);
    draw_text_centered_in_box(
        label,
        rect.x + 5.0,
        rect.y + 5.0,
        rect.w - 10.0,
        rect.h - 10.0,
        12.0,
        INK,
    );
}

pub(super) fn draw_mission_status(rect: Rect, unlocked: bool, lock_label: &str) {
    let label = if unlocked { "Available" } else { lock_label };
    draw_panel_with_fill(
        rect,
        if unlocked {
            Color::new(0.06, 0.18, 0.08, 0.92)
        } else {
            Color::new(0.13, 0.12, 0.09, 0.92)
        },
        false,
    );
    if unlocked {
        draw_check(vec2(rect.x + 12.0, rect.y + rect.h * 0.5));
    } else {
        draw_lock(vec2(rect.x + 12.0, rect.y + rect.h * 0.5), 0.72);
    }
    draw_text_centered_in_box(
        label,
        rect.x + 24.0,
        rect.y + 4.0,
        rect.w - 28.0,
        rect.h - 8.0,
        11.0,
        if unlocked {
            Color::new(0.58, 0.82, 0.36, 1.0)
        } else {
            MUTED
        },
    );
}

pub(super) fn draw_reward(pos: Vec2, reward: i64, font_size: f32) {
    draw_coin_icon(vec2(pos.x, pos.y - 5.0), 0.55);
    draw_ui_text_ex(
        &format!("{} Gold", reward),
        pos.x + 20.0,
        pos.y,
        TextStyle::new(font_size, INK).params(),
    );
}

fn draw_coin_icon(pos: Vec2, scale: f32) {
    draw_circle(pos.x, pos.y, 11.0 * scale, UI_GOLD);
    draw_circle_lines(
        pos.x,
        pos.y,
        11.0 * scale,
        1.5 * scale,
        Color::new(0.34, 0.20, 0.03, 1.0),
    );
    draw_circle_lines(
        pos.x,
        pos.y,
        6.0 * scale,
        1.0 * scale,
        Color::new(0.98, 0.82, 0.34, 1.0),
    );
}

pub(super) fn draw_meter_bar(rect: Rect, ratio: f32, fill: Color) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.02, 0.025, 0.02, 0.90),
    );
    draw_rectangle(rect.x, rect.y, rect.w * ratio.clamp(0.0, 1.0), rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, GOLD_SOFT);
}

pub(super) fn draw_route_icon(pos: Vec2, active: bool) {
    let color = if active { UI_GOLD } else { MUTED };
    draw_circle_lines(pos.x, pos.y, 12.0, 2.0, color);
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::TAU / 6.0;
        draw_line(
            pos.x,
            pos.y,
            pos.x + angle.cos() * 12.0,
            pos.y + angle.sin() * 12.0,
            1.5,
            color,
        );
    }
}

pub(super) fn draw_cargo_icon(pos: Vec2) {
    draw_rectangle(
        pos.x - 10.0,
        pos.y - 9.0,
        20.0,
        18.0,
        Color::new(0.52, 0.38, 0.18, 1.0),
    );
    draw_rectangle_lines(pos.x - 10.0, pos.y - 9.0, 20.0, 18.0, 2.0, UI_GOLD);
    draw_line(
        pos.x - 6.0,
        pos.y - 2.0,
        pos.x + 6.0,
        pos.y - 2.0,
        2.0,
        UI_GOLD,
    );
}

pub(super) fn draw_mini_icon(pos: Vec2, value: &str, hazard: bool) {
    if hazard {
        draw_hazard_icon(pos, value);
    } else {
        draw_threat_icon(pos, value);
    }
}

pub(super) fn draw_star(pos: Vec2, radius: f32, color: Color) {
    for i in 0..5 {
        let angle = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / 5.0;
        draw_line(
            pos.x,
            pos.y,
            pos.x + angle.cos() * radius,
            pos.y + angle.sin() * radius,
            2.0,
            color,
        );
    }
}

fn thumbnail_background(mission_type: &str) -> Color {
    match mission_type {
        "time_delivery" => Color::new(0.14, 0.105, 0.060, 1.0),
        "medicine_run" => Color::new(0.06, 0.16, 0.13, 1.0),
        "gold_shipment" => Color::new(0.18, 0.12, 0.035, 1.0),
        "princess_escort" | "royal_banquet_supplies" => Color::new(0.13, 0.09, 0.16, 1.0),
        "siege_supply_run" | "prisoner_escort" => Color::new(0.14, 0.105, 0.075, 1.0),
        _ => Color::new(0.055, 0.12, 0.08, 1.0),
    }
}

fn mission_type_style(mission_type: &str) -> (&'static str, Color) {
    match mission_type {
        "prisoner_escort" => ("Prisoner", Color::new(0.42, 0.18, 0.09, 0.94)),
        "princess_escort" => ("Royal", Color::new(0.26, 0.12, 0.38, 0.94)),
        "medicine_run" => ("Medicine", Color::new(0.08, 0.24, 0.34, 0.94)),
        "gold_shipment" => ("Gold", Color::new(0.42, 0.27, 0.02, 0.94)),
        "monster_egg_transport" => ("Egg", Color::new(0.18, 0.28, 0.12, 0.94)),
        "refugee_escort" => ("Refugees", Color::new(0.10, 0.18, 0.32, 0.94)),
        "royal_banquet_supplies" => ("Banquet", Color::new(0.30, 0.15, 0.32, 0.94)),
        "siege_supply_run" => ("Siege", Color::new(0.38, 0.12, 0.12, 0.94)),
        "time_delivery" => ("Timer", Color::new(0.25, 0.12, 0.34, 0.94)),
        _ => ("Cargo", Color::new(0.08, 0.26, 0.12, 0.94)),
    }
}

fn draw_forest_track(rect: Rect, order: u32) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.045, 0.095, 0.065, 1.0),
    );
    for i in 0..4 {
        let x = rect.x + 8.0 + i as f32 * rect.w * 0.24;
        draw_tree(vec2(x, rect.y + 16.0 + (i % 2) as f32 * 18.0), 0.7);
    }
    draw_triangle(
        vec2(rect.x + rect.w * 0.48, rect.bottom()),
        vec2(rect.x + rect.w * 0.22, rect.bottom()),
        vec2(rect.x + rect.w * 0.60, rect.y + 12.0),
        Color::new(0.43, 0.36, 0.22, 1.0),
    );
    draw_triangle(
        vec2(rect.x + rect.w * 0.48, rect.bottom()),
        vec2(rect.x + rect.w * 0.82, rect.bottom()),
        vec2(rect.x + rect.w * 0.60, rect.y + 12.0),
        Color::new(0.30, 0.25, 0.16, 1.0),
    );
    if order > 1 {
        draw_circle(
            rect.x + rect.w - 16.0,
            rect.y + 16.0,
            9.0,
            Color::new(0.68, 0.62, 0.48, 0.5),
        );
    }
}

fn draw_tree(pos: Vec2, scale: f32) {
    draw_rectangle(
        pos.x - 2.0 * scale,
        pos.y + 8.0 * scale,
        4.0 * scale,
        22.0 * scale,
        Color::new(0.12, 0.07, 0.04, 1.0),
    );
    draw_triangle(
        vec2(pos.x, pos.y - 10.0 * scale),
        vec2(pos.x - 14.0 * scale, pos.y + 18.0 * scale),
        vec2(pos.x + 14.0 * scale, pos.y + 18.0 * scale),
        Color::new(0.08, 0.23, 0.12, 1.0),
    );
}

fn draw_hourglass(rect: Rect) {
    draw_rectangle_lines(
        rect.x + 14.0,
        rect.y + 4.0,
        rect.w - 28.0,
        rect.h - 8.0,
        3.0,
        GOLD_SOFT,
    );
    draw_line(
        rect.x + 18.0,
        rect.y + 8.0,
        rect.right() - 18.0,
        rect.bottom() - 8.0,
        3.0,
        INK,
    );
    draw_line(
        rect.right() - 18.0,
        rect.y + 8.0,
        rect.x + 18.0,
        rect.bottom() - 8.0,
        3.0,
        INK,
    );
    draw_triangle(
        vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.52),
        vec2(rect.x + 24.0, rect.y + 16.0),
        vec2(rect.right() - 24.0, rect.y + 16.0),
        UI_GOLD,
    );
    draw_triangle(
        vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.52),
        vec2(rect.x + 24.0, rect.bottom() - 16.0),
        vec2(rect.right() - 24.0, rect.bottom() - 16.0),
        Color::new(0.70, 0.46, 0.18, 1.0),
    );
}

fn draw_prison_wagon(rect: Rect) {
    draw_rectangle(
        rect.x + 8.0,
        rect.y + 16.0,
        rect.w - 16.0,
        rect.h - 28.0,
        Color::new(0.19, 0.14, 0.10, 1.0),
    );
    for i in 0..4 {
        let x = rect.x + 18.0 + i as f32 * 12.0;
        draw_line(
            x,
            rect.y + 20.0,
            x,
            rect.bottom() - 20.0,
            3.0,
            Color::new(0.58, 0.50, 0.36, 1.0),
        );
    }
    draw_circle(
        rect.x + 18.0,
        rect.bottom() - 10.0,
        8.0,
        Color::new(0.05, 0.04, 0.03, 1.0),
    );
    draw_circle(
        rect.right() - 18.0,
        rect.bottom() - 10.0,
        8.0,
        Color::new(0.05, 0.04, 0.03, 1.0),
    );
}

fn draw_medicine_bottle(rect: Rect) {
    draw_rectangle(
        rect.x + rect.w * 0.42,
        rect.y + 8.0,
        rect.w * 0.16,
        18.0,
        Color::new(0.70, 0.82, 0.70, 1.0),
    );
    draw_circle(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.58,
        rect.w * 0.28,
        Color::new(0.58, 0.78, 0.58, 0.95),
    );
    draw_rectangle(
        rect.x + rect.w * 0.46,
        rect.y + rect.h * 0.40,
        rect.w * 0.08,
        rect.h * 0.34,
        Color::new(0.92, 0.94, 0.82, 1.0),
    );
    draw_rectangle(
        rect.x + rect.w * 0.34,
        rect.y + rect.h * 0.52,
        rect.w * 0.32,
        rect.h * 0.08,
        Color::new(0.92, 0.94, 0.82, 1.0),
    );
}

fn draw_gold_stacks(rect: Rect) {
    for i in 0..4 {
        let x = rect.x + 18.0 + i as f32 * 16.0;
        for j in 0..(3 + i % 2) {
            let y = rect.bottom() - 12.0 - j as f32 * 7.0;
            draw_ellipse(x, y, 13.0, 4.5, 0.0, UI_GOLD);
            draw_ellipse_lines(x, y, 13.0, 4.5, 0.0, 1.0, Color::new(0.32, 0.20, 0.04, 1.0));
        }
    }
}

fn draw_monster_egg(rect: Rect) {
    draw_ellipse(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.54,
        rect.w * 0.23,
        rect.h * 0.36,
        0.0,
        Color::new(0.68, 0.76, 0.58, 1.0),
    );
    draw_circle(
        rect.x + rect.w * 0.43,
        rect.y + rect.h * 0.42,
        4.0,
        Color::new(0.20, 0.44, 0.30, 1.0),
    );
    draw_circle(
        rect.x + rect.w * 0.56,
        rect.y + rect.h * 0.58,
        5.0,
        Color::new(0.20, 0.44, 0.30, 1.0),
    );
    draw_circle(
        rect.x + rect.w * 0.50,
        rect.y + rect.h * 0.72,
        3.0,
        Color::new(0.20, 0.44, 0.30, 1.0),
    );
}

fn draw_refugee_wagon(rect: Rect) {
    draw_rectangle(
        rect.x + 14.0,
        rect.y + 30.0,
        rect.w - 28.0,
        24.0,
        Color::new(0.42, 0.27, 0.13, 1.0),
    );
    draw_circle(
        rect.x + 24.0,
        rect.bottom() - 14.0,
        8.0,
        Color::new(0.06, 0.04, 0.025, 1.0),
    );
    draw_circle(
        rect.right() - 24.0,
        rect.bottom() - 14.0,
        8.0,
        Color::new(0.06, 0.04, 0.025, 1.0),
    );
    for i in 0..3 {
        draw_circle(
            rect.x + 24.0 + i as f32 * 16.0,
            rect.y + 24.0,
            6.0,
            Color::new(0.76, 0.64, 0.46, 1.0),
        );
    }
}

fn draw_crown(rect: Rect) {
    draw_triangle(
        vec2(rect.x + 12.0, rect.bottom() - 18.0),
        vec2(rect.x + 24.0, rect.y + 18.0),
        vec2(rect.x + 36.0, rect.bottom() - 18.0),
        UI_GOLD,
    );
    draw_triangle(
        vec2(rect.x + 30.0, rect.bottom() - 18.0),
        vec2(rect.x + rect.w * 0.5, rect.y + 8.0),
        vec2(rect.right() - 30.0, rect.bottom() - 18.0),
        UI_GOLD,
    );
    draw_triangle(
        vec2(rect.right() - 36.0, rect.bottom() - 18.0),
        vec2(rect.right() - 24.0, rect.y + 18.0),
        vec2(rect.right() - 12.0, rect.bottom() - 18.0),
        UI_GOLD,
    );
    draw_rectangle(
        rect.x + 14.0,
        rect.bottom() - 22.0,
        rect.w - 28.0,
        14.0,
        Color::new(0.42, 0.26, 0.08, 1.0),
    );
}

fn draw_banquet(rect: Rect) {
    draw_circle(
        rect.x + rect.w * 0.56,
        rect.y + rect.h * 0.62,
        18.0,
        Color::new(0.72, 0.58, 0.34, 1.0),
    );
    draw_circle(rect.x + rect.w * 0.45, rect.y + rect.h * 0.58, 9.0, UI_GOLD);
    draw_rectangle(
        rect.x + 17.0,
        rect.y + 16.0,
        10.0,
        38.0,
        Color::new(0.82, 0.78, 0.65, 1.0),
    );
    draw_ellipse(
        rect.x + 22.0,
        rect.y + 16.0,
        12.0,
        8.0,
        0.0,
        Color::new(0.82, 0.78, 0.65, 1.0),
    );
}

fn draw_catapult(rect: Rect) {
    draw_line(
        rect.x + 14.0,
        rect.bottom() - 14.0,
        rect.right() - 18.0,
        rect.y + 18.0,
        6.0,
        Color::new(0.42, 0.28, 0.14, 1.0),
    );
    draw_line(
        rect.x + 22.0,
        rect.bottom() - 16.0,
        rect.right() - 20.0,
        rect.bottom() - 16.0,
        4.0,
        Color::new(0.42, 0.28, 0.14, 1.0),
    );
    draw_circle(
        rect.x + 26.0,
        rect.bottom() - 12.0,
        8.0,
        Color::new(0.06, 0.04, 0.02, 1.0),
    );
    draw_circle(
        rect.right() - 28.0,
        rect.bottom() - 12.0,
        8.0,
        Color::new(0.06, 0.04, 0.02, 1.0),
    );
    draw_circle(
        rect.right() - 18.0,
        rect.y + 16.0,
        7.0,
        Color::new(0.24, 0.22, 0.20, 1.0),
    );
}

fn draw_hazard_icon(pos: Vec2, value: &str) {
    match value {
        "mud" => draw_ellipse(
            pos.x,
            pos.y + 2.0,
            14.0,
            6.0,
            0.0,
            Color::new(0.56, 0.38, 0.16, 1.0),
        ),
        "fallen_tree" => draw_line(
            pos.x - 12.0,
            pos.y + 5.0,
            pos.x + 12.0,
            pos.y - 6.0,
            5.0,
            Color::new(0.42, 0.24, 0.10, 1.0),
        ),
        "rocks" => {
            draw_circle(
                pos.x - 5.0,
                pos.y + 3.0,
                6.0,
                Color::new(0.52, 0.50, 0.44, 1.0),
            );
            draw_circle(
                pos.x + 6.0,
                pos.y - 3.0,
                8.0,
                Color::new(0.42, 0.42, 0.38, 1.0),
            );
        }
        "fire_patch" => draw_triangle(
            vec2(pos.x, pos.y - 12.0),
            vec2(pos.x - 10.0, pos.y + 10.0),
            vec2(pos.x + 10.0, pos.y + 10.0),
            Color::new(0.94, 0.38, 0.08, 1.0),
        ),
        _ => draw_circle(pos.x, pos.y, 8.0, UI_GOLD),
    }
}

fn draw_threat_icon(pos: Vec2, value: &str) {
    match value {
        "wolf" => {
            draw_circle(pos.x, pos.y, 8.0, Color::new(0.72, 0.72, 0.68, 1.0));
            draw_triangle(
                vec2(pos.x - 8.0, pos.y - 5.0),
                vec2(pos.x - 3.0, pos.y - 14.0),
                vec2(pos.x, pos.y - 4.0),
                Color::new(0.72, 0.72, 0.68, 1.0),
            );
            draw_triangle(
                vec2(pos.x + 8.0, pos.y - 5.0),
                vec2(pos.x + 3.0, pos.y - 14.0),
                vec2(pos.x, pos.y - 4.0),
                Color::new(0.72, 0.72, 0.68, 1.0),
            );
        }
        "bandit" | "bandit_archer" => {
            draw_line(pos.x - 9.0, pos.y + 8.0, pos.x + 9.0, pos.y - 8.0, 3.0, INK);
            draw_line(pos.x + 9.0, pos.y + 8.0, pos.x - 9.0, pos.y - 8.0, 3.0, INK);
        }
        "skeleton" | "necromancer" => {
            draw_circle(pos.x, pos.y, 8.0, Color::new(0.78, 0.82, 0.74, 1.0))
        }
        _ => draw_circle(pos.x, pos.y, 8.0, UI_GOLD),
    }
}

fn draw_check(pos: Vec2) {
    draw_circle_lines(pos.x, pos.y, 7.0, 2.0, Color::new(0.46, 0.78, 0.28, 1.0));
    draw_line(
        pos.x - 5.0,
        pos.y,
        pos.x - 2.0,
        pos.y + 4.0,
        2.0,
        Color::new(0.46, 0.78, 0.28, 1.0),
    );
    draw_line(
        pos.x - 2.0,
        pos.y + 4.0,
        pos.x + 5.0,
        pos.y - 5.0,
        2.0,
        Color::new(0.46, 0.78, 0.28, 1.0),
    );
}

fn draw_lock(pos: Vec2, scale: f32) {
    draw_rectangle(
        pos.x - 7.0 * scale,
        pos.y - 1.0 * scale,
        14.0 * scale,
        10.0 * scale,
        MUTED,
    );
    draw_circle_lines(pos.x, pos.y - 2.0 * scale, 7.0 * scale, 2.0 * scale, MUTED);
}
