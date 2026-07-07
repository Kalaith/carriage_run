//! Decorative drawing helpers for the carriage upgrade screen.

use crate::state::CarriageEquipment;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

pub(super) const GOLD: Color = Color::new(0.92, 0.66, 0.24, 1.0);
pub(super) const GOLD_SOFT: Color = Color::new(0.78, 0.56, 0.25, 0.72);
pub(super) const PANEL: Color = Color::new(0.055, 0.066, 0.056, 0.96);
pub(super) const PANEL_ALT: Color = Color::new(0.075, 0.086, 0.072, 0.97);
pub(super) const INK: Color = Color::new(0.98, 0.91, 0.70, 1.0);
pub(super) const MUTED: Color = Color::new(0.72, 0.78, 0.68, 1.0);

pub(super) fn draw_upgrade_backdrop() {
    draw_rectangle(
        0.0,
        0.0,
        super::LOGICAL_WIDTH,
        super::LOGICAL_HEIGHT,
        Color::new(0.018, 0.034, 0.030, 1.0),
    );
    draw_rectangle(
        0.0,
        0.0,
        super::LOGICAL_WIDTH,
        super::LOGICAL_HEIGHT,
        Color::new(0.025, 0.060, 0.048, 0.82),
    );
    draw_rectangle(
        812.0,
        0.0,
        210.0,
        super::LOGICAL_HEIGHT,
        Color::new(0.25, 0.18, 0.10, 0.82),
    );
    draw_rectangle(
        890.0,
        0.0,
        46.0,
        super::LOGICAL_HEIGHT,
        Color::new(0.44, 0.34, 0.18, 0.62),
    );
    for i in 0..16 {
        let x = if i % 2 == 0 {
            28.0 + i as f32 * 31.0
        } else {
            1068.0 + (i % 5) as f32 * 34.0
        };
        let y = 96.0 + (i as f32 * 43.0) % 560.0;
        draw_tree_shadow(vec2(x, y), 0.85 + (i % 3) as f32 * 0.18);
    }
    draw_wagon_silhouette(vec2(1110.0, 484.0));
    draw_rectangle(
        0.0,
        0.0,
        super::LOGICAL_WIDTH,
        super::LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.22),
    );
}

pub(super) fn draw_crest(rect: Rect) {
    draw_panel(rect, true);
    draw_triangle(
        vec2(rect.x + rect.w * 0.5, rect.bottom() - 6.0),
        vec2(rect.x + 16.0, rect.y + 18.0),
        vec2(rect.right() - 16.0, rect.y + 18.0),
        Color::new(0.07, 0.20, 0.16, 0.96),
    );
    draw_rectangle(
        rect.x + 24.0,
        rect.y + 26.0,
        rect.w - 48.0,
        34.0,
        Color::new(0.46, 0.27, 0.12, 1.0),
    );
    draw_circle(
        rect.x + 29.0,
        rect.y + 64.0,
        7.0,
        Color::new(0.09, 0.05, 0.025, 1.0),
    );
    draw_circle(
        rect.right() - 29.0,
        rect.y + 64.0,
        7.0,
        Color::new(0.09, 0.05, 0.025, 1.0),
    );
    draw_line(
        rect.x + 34.0,
        rect.y + 35.0,
        rect.right() - 34.0,
        rect.y + 35.0,
        3.0,
        GOLD,
    );
}

pub(super) fn draw_stat_icon(icon: &str, pos: Vec2) {
    match icon {
        "gold" => draw_coin_stack(pos, 0.55),
        "slots" => {
            draw_rectangle(
                pos.x - 12.0,
                pos.y - 9.0,
                24.0,
                18.0,
                Color::new(0.48, 0.32, 0.14, 1.0),
            );
            draw_rectangle_lines(pos.x - 12.0, pos.y - 9.0, 24.0, 18.0, 2.0, GOLD);
        }
        _ => {
            draw_poly(
                pos.x,
                pos.y,
                6,
                15.0,
                0.0,
                Color::new(0.20, 0.22, 0.20, 1.0),
            );
            draw_text_centered_in_box("1", pos.x - 12.0, pos.y - 12.0, 24.0, 24.0, 18.0, INK);
        }
    }
}

pub(super) fn nav_tile(rect: Rect, label: &str, icon: &str, active: bool, mouse: Vec2) -> bool {
    let hovered = rect.contains_point(mouse);
    let fill = if active {
        Color::new(0.10, 0.22, 0.12, 0.98)
    } else if hovered {
        Color::new(0.09, 0.11, 0.09, 0.98)
    } else {
        PANEL
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(fill).with_border(
            1.0,
            if active {
                Color::new(0.98, 0.73, 0.28, 1.0)
            } else {
                GOLD_SOFT
            },
        ),
    );
    draw_corner_marks(rect, if active { GOLD } else { GOLD_SOFT });
    draw_nav_icon(icon, vec2(rect.x + rect.w * 0.5, rect.y + 27.0));
    draw_text_centered_in_box(
        label,
        rect.x + 6.0,
        rect.bottom() - 28.0,
        rect.w - 12.0,
        20.0,
        14.0,
        INK,
    );
    hovered && is_mouse_button_released(MouseButton::Left)
}

pub(super) fn footer_button(rect: Rect, label: &str, mouse: Vec2) -> bool {
    let hovered = rect.contains_point(mouse);
    draw_surface(
        rect,
        &SurfaceStyle::new(if hovered {
            Color::new(0.12, 0.10, 0.075, 0.98)
        } else {
            PANEL
        })
        .with_border(1.0, GOLD_SOFT),
    );
    draw_corner_marks(rect, GOLD_SOFT);
    draw_text_centered_in_box(
        label,
        rect.x + 10.0,
        rect.y + 8.0,
        rect.w - 20.0,
        20.0,
        15.0,
        INK,
    );
    hovered && is_mouse_button_released(MouseButton::Left)
}

pub(super) fn gold_button(rect: Rect, label: &str, enabled: bool, mouse: Vec2) -> bool {
    let hovered = enabled && rect.contains_point(mouse);
    draw_surface(
        rect,
        &SurfaceStyle::new(if !enabled {
            Color::new(0.11, 0.12, 0.10, 0.78)
        } else if hovered {
            Color::new(0.08, 0.25, 0.13, 0.96)
        } else {
            Color::new(0.06, 0.18, 0.10, 0.96)
        })
        .with_border(
            1.0,
            if enabled {
                GOLD
            } else {
                Color::new(0.40, 0.34, 0.22, 0.65)
            },
        ),
    );
    draw_coin_stack(vec2(rect.x + 28.0, rect.y + rect.h * 0.5 + 1.0), 0.42);
    draw_text_centered_in_box(
        label,
        rect.x + 44.0,
        rect.y + 6.0,
        rect.w - 54.0,
        rect.h - 12.0,
        15.0,
        if enabled { INK } else { MUTED },
    );
    hovered && is_mouse_button_released(MouseButton::Left)
}

pub(super) fn small_close(rect: Rect, mouse: Vec2) -> bool {
    let hovered = rect.contains_point(mouse);
    draw_surface(
        rect,
        &SurfaceStyle::new(if hovered {
            Color::new(0.22, 0.13, 0.10, 1.0)
        } else {
            Color::new(0.12, 0.10, 0.08, 1.0)
        })
        .with_border(1.0, GOLD_SOFT),
    );
    draw_line(
        rect.x + 6.0,
        rect.y + 6.0,
        rect.right() - 6.0,
        rect.bottom() - 6.0,
        2.0,
        INK,
    );
    draw_line(
        rect.right() - 6.0,
        rect.y + 6.0,
        rect.x + 6.0,
        rect.bottom() + -6.0,
        2.0,
        INK,
    );
    hovered && is_mouse_button_released(MouseButton::Left)
}

pub(super) fn draw_section_title(label: &str, y: f32) {
    draw_line(304.0, y, 502.0, y, 1.0, GOLD_SOFT);
    draw_line(778.0, y, 976.0, y, 1.0, GOLD_SOFT);
    draw_text_centered_in_box(label, 510.0, y - 15.0, 260.0, 26.0, 22.0, GOLD);
}

pub(super) fn draw_section_label(label: &str, x: f32, y: f32, width: f32) {
    draw_line(x, y, x + 120.0, y, 1.0, GOLD_SOFT);
    draw_line(x + width - 120.0, y, x + width, y, 1.0, GOLD_SOFT);
    draw_text_centered_in_box(label, x + 124.0, y - 15.0, width - 248.0, 26.0, 18.0, GOLD);
}

pub(super) fn draw_panel(rect: Rect, strong: bool) {
    draw_panel_with_fill(
        rect,
        if strong {
            Color::new(0.08, 0.12, 0.10, 0.98)
        } else {
            PANEL
        },
        strong,
    );
}

pub(super) fn draw_panel_with_fill(rect: Rect, fill: Color, strong: bool) {
    let border = if strong { GOLD } else { GOLD_SOFT };
    draw_surface(
        rect,
        &SurfaceStyle::new(fill)
            .with_border(1.0, border)
            .with_top_highlight(
                2.0,
                Color::new(0.98, 0.70, 0.28, if strong { 0.55 } else { 0.34 }),
            ),
    );
    draw_corner_marks(rect, border);
}

pub(super) fn draw_upgrade_icon(id: &str, pos: Vec2, scale: f32) {
    match id {
        "carriage_armor" => draw_shield_icon(pos, scale),
        "reinforced_wheels" => draw_wheel_icon(pos, scale),
        "cargo_straps" => draw_straps_icon(pos, scale),
        "mounted_archer" => draw_quiver_icon(pos, scale),
        "guard_training" => draw_helmet_icon(pos, scale),
        "repair_kit" => draw_hammer_icon(pos, scale),
        "spiked_hubs" => draw_spikes_icon(pos, scale),
        "warding_lantern" => draw_lantern_icon(pos, scale),
        _ => draw_box_icon(pos, scale),
    }
}

pub(super) fn draw_equipment_icon(equipment: CarriageEquipment, pos: Vec2, scale: f32) {
    match equipment {
        CarriageEquipment::IronPlating => draw_shield_icon(pos, scale),
        CarriageEquipment::ReinforcedWheels => draw_wheel_icon(pos, scale),
        CarriageEquipment::CargoStraps => draw_straps_icon(pos, scale),
        CarriageEquipment::RepairKit => draw_hammer_icon(pos, scale),
        CarriageEquipment::SpikedHubs => draw_spikes_icon(pos, scale),
        CarriageEquipment::WardingLantern => draw_lantern_icon(pos, scale),
    }
}

fn draw_spikes_icon(pos: Vec2, scale: f32) {
    let hub = Color::new(0.22, 0.14, 0.08, 1.0);
    let spike = Color::new(0.82, 0.84, 0.80, 1.0);
    draw_circle(pos.x, pos.y, 10.0 * scale, hub);
    draw_circle_lines(pos.x, pos.y, 10.0 * scale, 2.0 * scale, spike);
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let (sin, cos) = angle.sin_cos();
        let base = vec2(pos.x + cos * 10.0 * scale, pos.y + sin * 10.0 * scale);
        let tip = vec2(pos.x + cos * 19.0 * scale, pos.y + sin * 19.0 * scale);
        let side = vec2(-sin, cos) * 3.0 * scale;
        draw_triangle(base + side, base - side, tip, spike);
    }
}

fn draw_lantern_icon(pos: Vec2, scale: f32) {
    let frame = Color::new(0.30, 0.24, 0.14, 1.0);
    let glass = Color::new(1.0, 0.86, 0.42, 0.92);
    draw_line(
        pos.x,
        pos.y - 18.0 * scale,
        pos.x,
        pos.y - 12.0 * scale,
        2.0 * scale,
        frame,
    );
    draw_rectangle(
        pos.x - 9.0 * scale,
        pos.y - 12.0 * scale,
        18.0 * scale,
        6.0 * scale,
        frame,
    );
    draw_circle(pos.x, pos.y + 2.0 * scale, 11.0 * scale, glass);
    draw_circle(
        pos.x,
        pos.y + 2.0 * scale,
        5.0 * scale,
        Color::new(1.0, 0.97, 0.8, 1.0),
    );
    draw_rectangle(
        pos.x - 11.0 * scale,
        pos.y + 12.0 * scale,
        22.0 * scale,
        5.0 * scale,
        frame,
    );
}

fn draw_corner_marks(rect: Rect, color: Color) {
    let len = 14.0;
    draw_line(rect.x, rect.y + len, rect.x + len, rect.y, 1.0, color);
    draw_line(
        rect.right() - len,
        rect.y,
        rect.right(),
        rect.y + len,
        1.0,
        color,
    );
    draw_line(
        rect.x,
        rect.bottom() - len,
        rect.x + len,
        rect.bottom(),
        1.0,
        color,
    );
    draw_line(
        rect.right() - len,
        rect.bottom(),
        rect.right(),
        rect.bottom() - len,
        1.0,
        color,
    );
}

fn draw_tree_shadow(pos: Vec2, scale: f32) {
    draw_rectangle(
        pos.x - 5.0 * scale,
        pos.y + 16.0 * scale,
        10.0 * scale,
        56.0 * scale,
        Color::new(0.025, 0.025, 0.018, 0.64),
    );
    draw_triangle(
        vec2(pos.x, pos.y - 38.0 * scale),
        vec2(pos.x - 34.0 * scale, pos.y + 34.0 * scale),
        vec2(pos.x + 34.0 * scale, pos.y + 34.0 * scale),
        Color::new(0.025, 0.075, 0.046, 0.70),
    );
    draw_triangle(
        vec2(pos.x, pos.y - 72.0 * scale),
        vec2(pos.x - 28.0 * scale, pos.y - 10.0 * scale),
        vec2(pos.x + 28.0 * scale, pos.y - 10.0 * scale),
        Color::new(0.020, 0.058, 0.038, 0.70),
    );
}

fn draw_wagon_silhouette(pos: Vec2) {
    draw_rectangle(
        pos.x - 62.0,
        pos.y - 48.0,
        124.0,
        76.0,
        Color::new(0.10, 0.065, 0.036, 0.90),
    );
    draw_rectangle(
        pos.x - 48.0,
        pos.y - 82.0,
        96.0,
        42.0,
        Color::new(0.13, 0.088, 0.046, 0.90),
    );
    draw_circle(
        pos.x - 50.0,
        pos.y + 34.0,
        18.0,
        Color::new(0.035, 0.025, 0.018, 0.96),
    );
    draw_circle(
        pos.x + 50.0,
        pos.y + 34.0,
        18.0,
        Color::new(0.035, 0.025, 0.018, 0.96),
    );
    draw_circle_lines(
        pos.x - 50.0,
        pos.y + 34.0,
        20.0,
        2.0,
        Color::new(0.64, 0.43, 0.18, 0.42),
    );
    draw_circle_lines(
        pos.x + 50.0,
        pos.y + 34.0,
        20.0,
        2.0,
        Color::new(0.64, 0.43, 0.18, 0.42),
    );
    draw_rectangle(
        pos.x - 15.0,
        pos.y - 4.0,
        30.0,
        42.0,
        Color::new(0.55, 0.34, 0.13, 0.80),
    );
}

fn draw_nav_icon(id: &str, pos: Vec2) {
    match id {
        "map" => {
            draw_rectangle(
                pos.x - 18.0,
                pos.y - 12.0,
                14.0,
                24.0,
                Color::new(0.40, 0.32, 0.20, 1.0),
            );
            draw_rectangle(
                pos.x - 4.0,
                pos.y - 9.0,
                14.0,
                24.0,
                Color::new(0.50, 0.40, 0.24, 1.0),
            );
            draw_rectangle(
                pos.x + 10.0,
                pos.y - 12.0,
                14.0,
                24.0,
                Color::new(0.40, 0.32, 0.20, 1.0),
            );
        }
        "shop" => draw_coin_stack(pos, 0.85),
        "guards" => draw_shield_icon(pos, 0.58),
        "up" => {
            draw_triangle(
                vec2(pos.x, pos.y - 20.0),
                vec2(pos.x - 18.0, pos.y + 6.0),
                vec2(pos.x + 18.0, pos.y + 6.0),
                GOLD,
            );
            draw_rectangle(pos.x - 7.0, pos.y + 4.0, 14.0, 20.0, GOLD);
        }
        _ => {
            draw_circle_lines(pos.x, pos.y, 18.0, 5.0, Color::new(0.65, 0.58, 0.46, 1.0));
            draw_circle(pos.x, pos.y, 6.0, Color::new(0.65, 0.58, 0.46, 1.0));
        }
    }
}

fn draw_shield_icon(pos: Vec2, scale: f32) {
    let s = 30.0 * scale;
    draw_triangle(
        vec2(pos.x, pos.y + s * 0.95),
        vec2(pos.x - s * 0.72, pos.y - s * 0.42),
        vec2(pos.x + s * 0.72, pos.y - s * 0.42),
        Color::new(0.72, 0.72, 0.66, 1.0),
    );
    draw_triangle(
        vec2(pos.x, pos.y + s * 0.66),
        vec2(pos.x - s * 0.48, pos.y - s * 0.25),
        vec2(pos.x + s * 0.48, pos.y - s * 0.25),
        Color::new(0.42, 0.45, 0.42, 1.0),
    );
    draw_line(
        pos.x,
        pos.y - s * 0.42,
        pos.x,
        pos.y + s * 0.70,
        2.0 * scale,
        Color::new(0.95, 0.90, 0.72, 1.0),
    );
}

fn draw_wheel_icon(pos: Vec2, scale: f32) {
    let r = 26.0 * scale;
    draw_circle_lines(
        pos.x,
        pos.y,
        r,
        5.0 * scale,
        Color::new(0.58, 0.42, 0.20, 1.0),
    );
    draw_circle(pos.x, pos.y, 6.0 * scale, GOLD);
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::PI / 4.0;
        draw_line(
            pos.x,
            pos.y,
            pos.x + angle.cos() * r,
            pos.y + angle.sin() * r,
            2.0 * scale,
            GOLD_SOFT,
        );
    }
}

fn draw_straps_icon(pos: Vec2, scale: f32) {
    let w = 34.0 * scale;
    let h = 48.0 * scale;
    draw_rectangle(
        pos.x - w * 0.5,
        pos.y - h * 0.45,
        w,
        h,
        Color::new(0.42, 0.24, 0.12, 1.0),
    );
    draw_rectangle(
        pos.x - w * 0.34,
        pos.y - h * 0.30,
        w * 0.68,
        h * 0.12,
        GOLD_SOFT,
    );
    draw_rectangle(
        pos.x - w * 0.34,
        pos.y + h * 0.14,
        w * 0.68,
        h * 0.12,
        GOLD_SOFT,
    );
    draw_line(
        pos.x - w * 0.50,
        pos.y - h * 0.44,
        pos.x + w * 0.50,
        pos.y + h * 0.46,
        3.0 * scale,
        Color::new(0.20, 0.11, 0.06, 1.0),
    );
}

fn draw_quiver_icon(pos: Vec2, scale: f32) {
    draw_straps_icon(vec2(pos.x - 2.0 * scale, pos.y + 6.0 * scale), scale * 0.72);
    for offset in [-12.0, 0.0, 12.0] {
        draw_line(
            pos.x + offset * scale,
            pos.y + 6.0 * scale,
            pos.x + (offset - 8.0) * scale,
            pos.y - 33.0 * scale,
            3.0 * scale,
            Color::new(0.82, 0.70, 0.44, 1.0),
        );
        draw_triangle(
            vec2(pos.x + (offset - 8.0) * scale, pos.y - 37.0 * scale),
            vec2(pos.x + (offset - 13.0) * scale, pos.y - 27.0 * scale),
            vec2(pos.x + (offset - 3.0) * scale, pos.y - 27.0 * scale),
            GOLD,
        );
    }
}

fn draw_helmet_icon(pos: Vec2, scale: f32) {
    let r = 26.0 * scale;
    draw_circle(pos.x, pos.y, r, Color::new(0.56, 0.55, 0.50, 1.0));
    draw_rectangle(
        pos.x - r,
        pos.y,
        r * 2.0,
        r * 0.82,
        Color::new(0.56, 0.55, 0.50, 1.0),
    );
    draw_rectangle(
        pos.x - r * 0.52,
        pos.y - r * 0.15,
        r * 0.30,
        r * 0.82,
        Color::new(0.20, 0.20, 0.18, 1.0),
    );
    draw_rectangle(
        pos.x + r * 0.18,
        pos.y - r * 0.15,
        r * 0.30,
        r * 0.82,
        Color::new(0.20, 0.20, 0.18, 1.0),
    );
    draw_line(pos.x - r, pos.y, pos.x + r, pos.y, 3.0 * scale, GOLD_SOFT);
}

fn draw_hammer_icon(pos: Vec2, scale: f32) {
    draw_line(
        pos.x - 20.0 * scale,
        pos.y + 24.0 * scale,
        pos.x + 18.0 * scale,
        pos.y - 14.0 * scale,
        7.0 * scale,
        Color::new(0.46, 0.28, 0.12, 1.0),
    );
    draw_rectangle(
        pos.x + 3.0 * scale,
        pos.y - 31.0 * scale,
        36.0 * scale,
        15.0 * scale,
        Color::new(0.58, 0.57, 0.52, 1.0),
    );
    draw_rectangle(
        pos.x + 26.0 * scale,
        pos.y - 21.0 * scale,
        10.0 * scale,
        24.0 * scale,
        Color::new(0.58, 0.57, 0.52, 1.0),
    );
}

fn draw_box_icon(pos: Vec2, scale: f32) {
    draw_rectangle(
        pos.x - 22.0 * scale,
        pos.y - 18.0 * scale,
        44.0 * scale,
        36.0 * scale,
        Color::new(0.42, 0.24, 0.12, 1.0),
    );
    draw_rectangle_lines(
        pos.x - 22.0 * scale,
        pos.y - 18.0 * scale,
        44.0 * scale,
        36.0 * scale,
        2.0 * scale,
        GOLD,
    );
}

fn draw_coin_stack(pos: Vec2, scale: f32) {
    let w = 24.0 * scale;
    let h = 7.0 * scale;
    for i in 0..4 {
        let y = pos.y + (3 - i) as f32 * h * 0.62;
        draw_ellipse(
            pos.x,
            y,
            w * 0.5,
            h * 0.55,
            0.0,
            Color::new(0.96, 0.70, 0.18, 1.0),
        );
        draw_ellipse_lines(
            pos.x,
            y,
            w * 0.5,
            h * 0.55,
            0.0,
            1.5 * scale,
            Color::new(0.40, 0.25, 0.06, 1.0),
        );
    }
}
