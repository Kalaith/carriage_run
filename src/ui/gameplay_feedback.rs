//! Mission feedback that is intentionally separate from the dense HUD module.

use super::upgrade_visuals::{draw_panel_with_fill, GOLD, GOLD_SOFT, INK, MUTED, PANEL};
use super::{touch_boost_rect, touch_brake_rect, touch_steer_left_rect, touch_steer_right_rect};
use crate::state::{Enemy, MissionRun};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::draw_text_centered_in_box;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_touch_controls(run: &MissionRun, colorblind_safe: bool) {
    let palette = crate::settings::colorblind_palette(colorblind_safe);
    let info = Color::new(palette[3].0, palette[3].1, palette[3].2, 0.96);
    let warning = Color::new(palette[1].0, palette[1].1, palette[1].2, 0.96);
    for (rect, label, color) in [
        (touch_steer_left_rect(), "STEER ◀", info),
        (touch_steer_right_rect(), "STEER ▶", info),
        (touch_brake_rect(), "BRAKE", info),
        (touch_boost_rect(), "BOOST", warning),
    ] {
        draw_panel_with_fill(rect, color, true);
        draw_text_centered_in_box(label, rect.x, rect.y + 18.0, rect.w, 28.0, 16.0, INK);
    }
    draw_ui_text_ex(
        "Touch controls",
        186.0,
        614.0,
        macroquad_toolkit::prelude::TextStyle::new(12.0, MUTED).params(),
    );
    if run.is_in_night_stretch() {
        draw_text_centered_in_box(
            "NIGHT — lane visibility reduced",
            440.0,
            644.0,
            260.0,
            24.0,
            13.0,
            Color::new(0.64, 0.76, 1.0, 1.0),
        );
    }
    draw_guided_first_route(run);
}

fn draw_guided_first_route(run: &MissionRun) {
    if run.mission_id != "muddy_road" || run.elapsed > 24.0 {
        return;
    }
    let lessons = [
        "Tap STEER ◀ or STEER ▶ to keep the wagon on the road.",
        "A WAVE INCOMING warning gives you time to prepare.",
        "Drag a guard on the road to issue a move order.",
        "Tap a guard to switch between Roam and Hold.",
        "Drag a ranged guard onto a gold wagon slot to mount them.",
        "Tap BRAKE to steady rough hazards and protect cargo.",
        "Hold BOOST when the road is clear to reach the marker faster.",
    ];
    let index = ((run.elapsed / 3.2).floor() as usize).min(lessons.len() - 1);
    let rect = Rect::new(340.0, 242.0, 600.0, 46.0);
    draw_panel_with_fill(rect, Color::new(0.05, 0.14, 0.16, 0.96), true);
    draw_text_centered_in_box(
        lessons[index],
        rect.x + 12.0,
        rect.y + 11.0,
        rect.w - 24.0,
        24.0,
        15.0,
        INK,
    );
}

pub(super) fn draw_boss_and_breakout(run: &MissionRun) {
    if let Some((name, phase, ratio)) = run.boss_status() {
        let rect = Rect::new(390.0, 150.0, 500.0, 42.0);
        draw_panel_with_fill(rect, Color::new(0.20, 0.045, 0.04, 0.94), true);
        draw_ui_text_ex(
            &format!("{} · {}", name, phase),
            rect.x + 14.0,
            rect.y + 18.0,
            macroquad_toolkit::prelude::TextStyle::new(14.0, GOLD).params(),
        );
        draw_rectangle(rect.x + 14.0, rect.y + 25.0, rect.w - 28.0, 8.0, PANEL);
        draw_rectangle(
            rect.x + 14.0,
            rect.y + 25.0,
            (rect.w - 28.0) * ratio,
            8.0,
            Color::new(0.86, 0.18, 0.16, 1.0),
        );
    }
    if let Some((progress, active)) = run.breakout_status() {
        let rect = Rect::new(364.0, 202.0, 552.0, 34.0);
        draw_panel_with_fill(rect, Color::new(0.20, 0.12, 0.04, 0.92), active);
        draw_ui_text_ex(
            if active {
                "BREAKOUT — BRAKE / GUARD"
            } else {
                "Prisoner security"
            },
            rect.x + 14.0,
            rect.y + 22.0,
            macroquad_toolkit::prelude::TextStyle::new(14.0, GOLD).params(),
        );
        draw_rectangle(rect.x + 190.0, rect.y + 12.0, 340.0, 9.0, PANEL);
        draw_rectangle(
            rect.x + 190.0,
            rect.y + 12.0,
            340.0 * progress,
            9.0,
            if active { GOLD } else { GOLD_SOFT },
        );
    }
}

pub(super) fn draw_offscreen_threat_pips(enemies: &[Enemy]) {
    for enemy in enemies.iter().filter(|enemy| enemy.is_active()) {
        let outside = enemy.pos.x < 250.0
            || enemy.pos.x > 1030.0
            || enemy.pos.y < 112.0
            || enemy.pos.y > 590.0;
        if !outside {
            continue;
        }
        let x = enemy.pos.x.clamp(264.0, 1016.0);
        let y = enemy.pos.y.clamp(118.0, 586.0);
        draw_circle(x, y, 7.0, Color::new(0.88, 0.28, 0.20, 0.96));
        draw_circle_lines(x, y, 10.0, 1.0, GOLD_SOFT);
        draw_triangle(
            vec2(x, y - 14.0),
            vec2(x - 5.0, y - 5.0),
            vec2(x + 5.0, y - 5.0),
            Color::new(0.96, 0.56, 0.26, 0.96),
        );
    }
}
