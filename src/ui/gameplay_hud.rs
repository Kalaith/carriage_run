//! Active mission HUD chrome.

use super::upgrade_visuals::{draw_panel_with_fill, GOLD, GOLD_SOFT, INK, MUTED, PANEL};
use super::{UiAction, UiContext, LOGICAL_WIDTH};
use crate::state::{Guard, GuardKind, MissionRun};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

const RED: Color = Color::new(0.78, 0.12, 0.10, 1.0);
const GREEN: Color = Color::new(0.24, 0.68, 0.30, 1.0);
const BAR_BG: Color = Color::new(0.045, 0.035, 0.025, 0.98);

pub(super) fn draw_gameplay_hud(
    ctx: &UiContext<'_>,
    run: &MissionRun,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_top_hud(run, mouse, actions);
    draw_bottom_hud(run);
    draw_speed_gauge(run);

    if let Some(journey) = &ctx.session.journey {
        draw_expedition_tag(journey.leg, journey.banked_gold);
    }

    if let Some(wave) = run.wave_telegraph() {
        draw_wave_telegraph(wave);
    }

    if ctx.session.campaign.alerts_enabled && run.alert.timer > 0.0 {
        let alpha = (run.alert.timer / 1.6).clamp(0.0, 1.0);
        draw_text_centered_in_box(
            &run.alert.text,
            492.0,
            106.0,
            296.0,
            34.0,
            22.0,
            Color::new(1.0, 0.86, 0.48, alpha),
        );
    }
}

fn draw_top_hud(run: &MissionRun, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let rect = Rect::new(8.0, 8.0, LOGICAL_WIDTH - 16.0, 86.0);
    draw_panel_with_fill(rect, Color::new(0.060, 0.043, 0.027, 0.98), true);
    draw_ornate_frame(rect);

    draw_status_meter(
        Rect::new(rect.x + 18.0, rect.y + 12.0, 260.0, 62.0),
        "Carriage Health",
        run.carriage.health,
        run.carriage.max_health,
        RED,
        "carriage",
    );
    draw_status_meter(
        Rect::new(rect.x + 290.0, rect.y + 12.0, 260.0, 62.0),
        "Cargo Integrity",
        run.carriage.cargo,
        run.carriage.max_cargo,
        Color::new(0.88, 0.60, 0.08, 1.0),
        "cargo",
    );
    draw_route_progress(Rect::new(rect.x + 562.0, rect.y + 14.0, 330.0, 58.0), run);

    if let (Some(label), Some(ratio)) = (run.mission_kind.label(), run.special_ratio()) {
        let meter = Rect::new(rect.x + 904.0, rect.y + 18.0, 82.0, 50.0);
        draw_small_special(meter, label, ratio);
        // Princess "drive clean" challenge: the live smoothness multiplier.
        if let Some(mult) = run.ride_smoothness_multiplier() {
            let color = if mult >= 1.7 {
                Color::new(0.42, 0.86, 0.46, 1.0)
            } else if mult >= 1.35 {
                Color::new(0.95, 0.82, 0.36, 1.0)
            } else {
                Color::new(0.95, 0.55, 0.42, 1.0)
            };
            draw_text_centered(
                &format!("Smooth x{:.2}", mult),
                meter.x + meter.w / 2.0,
                meter.bottom() + 16.0,
                TextStyle::new(15.0, color),
            );
        }
    }

    let (time_label, time_color) = timer_label(run);
    draw_timer_panel(
        Rect::new(rect.right() - 360.0, rect.y + 18.0, 104.0, 50.0),
        &time_label,
        time_color,
    );
    if hud_button(
        Rect::new(rect.right() - 244.0, rect.y + 14.0, 112.0, 58.0),
        "Repair",
        "repair",
        run.repair_available(),
        mouse,
    ) {
        actions.push(UiAction::UseRepair);
    }
    if hud_button(
        Rect::new(rect.right() - 120.0, rect.y + 14.0, 104.0, 58.0),
        "Pause",
        "pause",
        true,
        mouse,
    ) {
        actions.push(UiAction::PauseGame);
    }
}

fn draw_speed_gauge(run: &MissionRun) {
    let rect = Rect::new(14.0, 526.0, 162.0, 62.0);
    draw_divider_panel(rect);
    let (accent, state) = if run.is_slowed() {
        (Color::new(0.95, 0.62, 0.18, 1.0), "SLOWED")
    } else if run.is_boosted() {
        (GREEN, "BOOST")
    } else if run.is_braking() {
        (Color::new(0.52, 0.66, 0.98, 1.0), "BRAKE")
    } else {
        (Color::new(0.52, 0.78, 0.92, 1.0), "CRUISE")
    };
    draw_ui_text_ex(
        "SPEED",
        rect.x + 12.0,
        rect.y + 18.0,
        TextStyle::new(13.0, GOLD).params(),
    );
    draw_text_right(
        state,
        rect.right() - 12.0,
        rect.y + 18.0,
        TextStyle::new(12.0, accent),
    );
    draw_ui_text_ex(
        &format!("{:.0}", run.speed_readout()),
        rect.x + 12.0,
        rect.y + 43.0,
        TextStyle::new(24.0, INK).params(),
    );
    draw_ui_text_ex(
        "mph",
        rect.x + 50.0,
        rect.y + 43.0,
        TextStyle::new(13.0, MUTED).params(),
    );
    draw_meter_bar(
        Rect::new(rect.x + 12.0, rect.y + 49.0, rect.w - 24.0, 8.0),
        run.speed_ratio(),
        accent,
        "",
    );
}

fn draw_expedition_tag(leg: u32, banked: i64) {
    let rect = Rect::new(LOGICAL_WIDTH * 0.5 - 108.0, 100.0, 216.0, 26.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.06, 0.10, 0.06, 0.86),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, GOLD_SOFT);
    draw_text_centered_in_box(
        &format!("EXPEDITION  -  Leg {}  -  {}g banked", leg, banked),
        rect.x,
        rect.y + 5.0,
        rect.w,
        16.0,
        14.0,
        Color::new(0.86, 0.92, 0.72, 1.0),
    );
}

fn draw_wave_telegraph(wave: u32) {
    let rect = Rect::new(LOGICAL_WIDTH * 0.5 - 150.0, 150.0, 300.0, 42.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.20, 0.04, 0.03, 0.82),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        Color::new(0.95, 0.42, 0.24, 0.95),
    );
    draw_text_centered_in_box(
        &format!("! WAVE {} INCOMING !", wave),
        rect.x,
        rect.y + 8.0,
        rect.w,
        24.0,
        22.0,
        Color::new(1.0, 0.72, 0.40, 1.0),
    );
}

fn draw_bottom_hud(run: &MissionRun) {
    let deck = Rect::new(8.0, 598.0, LOGICAL_WIDTH - 16.0, 114.0);
    draw_panel_with_fill(deck, Color::new(0.045, 0.036, 0.026, 0.98), true);
    draw_ornate_frame(deck);

    let mut card_x = deck.x + 20.0;
    for guard in run.guards.iter().take(2) {
        draw_guard_card(Rect::new(card_x, deck.y + 16.0, 244.0, 82.0), guard);
        card_x += 260.0;
    }

    draw_command_panel(
        Rect::new(deck.x + 530.0, deck.y + 16.0, 330.0, 82.0),
        run.guards.len().saturating_sub(2),
    );
    draw_mission_panel(Rect::new(deck.x + 880.0, deck.y + 16.0, 360.0, 82.0), run);
}

fn draw_status_meter(rect: Rect, label: &str, value: f32, max: f32, fill: Color, icon: &str) {
    draw_divider_panel(rect);
    draw_round_icon(vec2(rect.x + 34.0, rect.y + 31.0), icon);
    draw_ui_text_ex(
        label,
        rect.x + 76.0,
        rect.y + 20.0,
        TextStyle::new(16.0, INK).params(),
    );
    draw_meter_bar(
        Rect::new(rect.x + 76.0, rect.y + 30.0, rect.w - 92.0, 25.0),
        value / max.max(1.0),
        fill,
        &format!("{:.0}%", value / max.max(1.0) * 100.0),
    );
}

fn draw_route_progress(rect: Rect, run: &MissionRun) {
    draw_divider_panel(rect);
    draw_text_centered_in_box(
        "Route Progress",
        rect.x,
        rect.y + 2.0,
        rect.w,
        18.0,
        15.0,
        INK,
    );
    draw_meter_bar(
        Rect::new(rect.x + 18.0, rect.y + 28.0, rect.w - 46.0, 22.0),
        run.progress_ratio(),
        Color::new(0.28, 0.65, 0.28, 1.0),
        &format!("{:.0}%", run.progress_ratio() * 100.0),
    );
    draw_flag(vec2(rect.right() - 18.0, rect.y + 40.0));
}

fn draw_small_special(rect: Rect, label: &str, ratio: f32) {
    draw_divider_panel(rect);
    draw_text_centered_in_box(
        label,
        rect.x + 6.0,
        rect.y + 6.0,
        rect.w - 12.0,
        16.0,
        11.0,
        MUTED,
    );
    draw_meter_bar(
        Rect::new(rect.x + 12.0, rect.y + 30.0, rect.w - 24.0, 12.0),
        ratio,
        Color::new(0.44, 0.56, 0.86, 1.0),
        "",
    );
}

fn draw_timer_panel(rect: Rect, label: &str, color: Color) {
    draw_divider_panel(rect);
    draw_hourglass(vec2(rect.x + 25.0, rect.y + 25.0), color);
    draw_ui_text_ex(
        label,
        rect.x + 48.0,
        rect.y + 34.0,
        TextStyle::new(22.0, INK).params(),
    );
}

fn hud_button(rect: Rect, label: &str, icon: &str, enabled: bool, mouse: Vec2) -> bool {
    let hovered = enabled && rect.contains_point(mouse);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let fill = if !enabled {
        Color::new(0.06, 0.06, 0.05, 0.74)
    } else if pressed {
        Color::new(0.04, 0.18, 0.08, 1.0)
    } else if hovered {
        Color::new(0.10, 0.28, 0.13, 1.0)
    } else if icon == "repair" {
        Color::new(0.06, 0.20, 0.10, 0.98)
    } else {
        PANEL
    };
    draw_panel_with_fill(rect, fill, enabled);
    draw_button_icon(icon, vec2(rect.x + 27.0, rect.y + 29.0), enabled);
    draw_text_centered_in_box(
        label,
        rect.x + 44.0,
        rect.y + 15.0,
        rect.w - 52.0,
        rect.h - 22.0,
        18.0,
        if enabled { INK } else { MUTED },
    );
    hovered && is_mouse_button_released(MouseButton::Left)
}

fn draw_guard_card(rect: Rect, guard: &Guard) {
    let active = guard.is_active();
    let fill = if active {
        Color::new(0.055, 0.064, 0.058, 0.98)
    } else {
        Color::new(0.050, 0.048, 0.046, 0.84)
    };
    draw_panel_with_fill(rect, fill, active);
    draw_guard_portrait(Rect::new(rect.x + 12.0, rect.y + 12.0, 58.0, 58.0), guard);

    draw_ui_text_ex(
        &format!("{} {}", guard.kind.label(), guard.star_level),
        rect.x + 82.0,
        rect.y + 25.0,
        TextStyle::new(18.0, INK).params(),
    );
    draw_ui_text_ex(
        &format!("{:.0}/{:.0}", guard.health, guard.max_health),
        rect.x + 100.0,
        rect.y + 44.0,
        TextStyle::new(15.0, INK).params(),
    );
    draw_heart(vec2(rect.x + 90.0, rect.y + 39.0), 5.0, RED);
    draw_meter_bar(
        Rect::new(rect.x + 82.0, rect.y + 48.0, rect.w - 98.0, 9.0),
        guard.health / guard.max_health.max(1.0),
        GREEN,
        "",
    );
    draw_role_badge(
        Rect::new(rect.x + 82.0, rect.y + 61.0, 68.0, 18.0),
        if !active {
            "Down"
        } else {
            guard.stance_label()
        },
        active,
    );
    draw_drag_hint(vec2(rect.x + 162.0, rect.y + 71.0));
}

fn draw_command_panel(rect: Rect, hidden_guards: usize) {
    draw_panel_with_fill(rect, Color::new(0.050, 0.046, 0.036, 0.96), false);
    draw_compass(vec2(rect.x + 54.0, rect.y + 44.0));
    draw_text_block(
        "Protect the carriage and deliver the cargo safely to the end of the route.",
        rect.x + 112.0,
        rect.y + 24.0,
        rect.w - 132.0,
        34.0,
        16.0,
        2.0,
        INK,
    );
    let hint = if hidden_guards > 0 {
        format!("Tap guard: Roam/Hold  |  ^ Boost  v Brake  |  {hidden_guards} more active")
    } else {
        "Tap guard: Roam/Hold  |  ^ Boost  v Brake".to_owned()
    };
    draw_ui_text_ex(
        &hint,
        rect.x + 112.0,
        rect.y + 72.0,
        TextStyle::new(12.0, MUTED).params(),
    );
}

fn draw_mission_panel(rect: Rect, run: &MissionRun) {
    draw_panel_with_fill(rect, Color::new(0.050, 0.043, 0.032, 0.98), false);
    draw_ui_text_ex(
        &run.mission_name.to_uppercase(),
        rect.x + 24.0,
        rect.y + 24.0,
        TextStyle::new(18.0, GOLD).params(),
    );
    draw_line(
        rect.x + 24.0,
        rect.y + 34.0,
        rect.right() - 26.0,
        rect.y + 34.0,
        1.0,
        GOLD_SOFT,
    );
    draw_objective_row(
        vec2(rect.x + 34.0, rect.y + 52.0),
        "Enemies Defeated",
        &run.enemies_defeated.to_string(),
    );
    draw_flag(vec2(rect.x + 36.0, rect.y + 74.0));
    draw_ui_text_ex(
        "Current Objective",
        rect.x + 58.0,
        rect.y + 70.0,
        TextStyle::new(15.0, INK).params(),
    );
    draw_ui_text_ex(
        &objective_text(run),
        rect.x + 58.0,
        rect.y + 90.0,
        TextStyle::new(14.0, Color::new(0.35, 0.88, 0.39, 1.0)).params(),
    );
}

fn draw_objective_row(pos: Vec2, label: &str, value: &str) {
    draw_skull(pos);
    draw_ui_text_ex(
        label,
        pos.x + 24.0,
        pos.y + 4.0,
        TextStyle::new(15.0, INK).params(),
    );
    draw_text_right(value, pos.x + 314.0, pos.y + 4.0, TextStyle::new(15.0, INK));
}

fn objective_text(run: &MissionRun) -> String {
    if let Some(limit) = run.time_limit {
        let remaining = (limit - run.elapsed).max(0.0);
        format!("Deliver before the timer expires. {:.0}s left.", remaining)
    } else if run.progress_ratio() > 0.86 {
        "Reach the final marker and protect the cargo.".to_owned()
    } else {
        "Reach the end of the road safely.".to_owned()
    }
}

fn draw_guard_portrait(rect: Rect, guard: &Guard) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.05, 0.08, 0.09, 1.0),
    );
    let color = guard_color(guard.kind);
    draw_circle(rect.x + 30.0, rect.y + 25.0, 20.0, color);
    draw_circle(
        rect.x + 30.0,
        rect.y + 20.0,
        12.0,
        Color::new(0.76, 0.56, 0.38, 1.0),
    );
    match guard.kind {
        GuardKind::Archer | GuardKind::CrossbowGuard => {
            draw_line(
                rect.x + 14.0,
                rect.y + 47.0,
                rect.x + 48.0,
                rect.y + 18.0,
                3.0,
                GOLD,
            );
        }
        GuardKind::Mage => draw_circle(
            rect.x + 46.0,
            rect.y + 18.0,
            6.0,
            Color::new(0.50, 0.86, 1.0, 1.0),
        ),
        _ => draw_line(
            rect.x + 44.0,
            rect.y + 46.0,
            rect.x + 52.0,
            rect.y + 14.0,
            4.0,
            INK,
        ),
    }
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, GOLD_SOFT);
}

fn guard_color(kind: GuardKind) -> Color {
    match kind {
        GuardKind::Swordsman => Color::new(0.14, 0.32, 0.58, 1.0),
        GuardKind::ShieldGuard => Color::new(0.18, 0.44, 0.30, 1.0),
        GuardKind::Spearman => Color::new(0.34, 0.28, 0.58, 1.0),
        GuardKind::Archer => Color::new(0.14, 0.38, 0.18, 1.0),
        GuardKind::CrossbowGuard => Color::new(0.36, 0.30, 0.24, 1.0),
        GuardKind::Mage => Color::new(0.20, 0.28, 0.62, 1.0),
    }
}

fn draw_meter_bar(rect: Rect, ratio: f32, fill: Color, label: &str) {
    let ratio = ratio.clamp(0.0, 1.0);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, BAR_BG);
    draw_rectangle(
        rect.x + 2.0,
        rect.y + 2.0,
        (rect.w - 4.0) * ratio,
        rect.h - 4.0,
        fill,
    );
    draw_rectangle(
        rect.x + 2.0,
        rect.y + 2.0,
        rect.w - 4.0,
        3.0,
        Color::new(1.0, 0.92, 0.64, 0.16),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, GOLD_SOFT);
    if !label.is_empty() {
        draw_text_centered_in_box(
            label,
            rect.x,
            rect.y + 2.0,
            rect.w,
            rect.h - 4.0,
            15.0,
            WHITE,
        );
    }
}

fn draw_divider_panel(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.045, 0.035, 0.026, 0.92),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(GOLD_SOFT.r, GOLD_SOFT.g, GOLD_SOFT.b, 0.42),
    );
}

fn draw_ornate_frame(rect: Rect) {
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, GOLD_SOFT);
    for (x, y, sx, sy) in [
        (rect.x, rect.y, 1.0, 1.0),
        (rect.right(), rect.y, -1.0, 1.0),
        (rect.x, rect.bottom(), 1.0, -1.0),
        (rect.right(), rect.bottom(), -1.0, -1.0),
    ] {
        draw_line(x, y, x + sx * 22.0, y, 3.0, GOLD);
        draw_line(x, y, x, y + sy * 22.0, 3.0, GOLD);
        draw_line(
            x + sx * 6.0,
            y + sy * 6.0,
            x + sx * 22.0,
            y + sy * 22.0,
            2.0,
            GOLD_SOFT,
        );
    }
}

fn draw_round_icon(pos: Vec2, icon: &str) {
    draw_circle(pos.x, pos.y, 26.0, Color::new(0.12, 0.10, 0.07, 1.0));
    draw_circle_lines(pos.x, pos.y, 26.0, 2.0, GOLD_SOFT);
    match icon {
        "cargo" => {
            draw_rectangle(
                pos.x - 12.0,
                pos.y - 8.0,
                24.0,
                18.0,
                Color::new(0.55, 0.34, 0.14, 1.0),
            );
            draw_line(pos.x - 12.0, pos.y - 8.0, pos.x, pos.y - 16.0, 2.0, GOLD);
            draw_line(pos.x + 12.0, pos.y - 8.0, pos.x, pos.y - 16.0, 2.0, GOLD);
        }
        _ => {
            draw_rectangle(
                pos.x - 14.0,
                pos.y - 9.0,
                28.0,
                18.0,
                Color::new(0.48, 0.29, 0.12, 1.0),
            );
            draw_circle(pos.x - 10.0, pos.y + 12.0, 5.0, BAR_BG);
            draw_circle(pos.x + 10.0, pos.y + 12.0, 5.0, BAR_BG);
        }
    }
}

fn draw_button_icon(icon: &str, pos: Vec2, enabled: bool) {
    let color = if enabled { GOLD } else { MUTED };
    if icon == "pause" {
        draw_rectangle(pos.x - 8.0, pos.y - 13.0, 6.0, 26.0, color);
        draw_rectangle(pos.x + 4.0, pos.y - 13.0, 6.0, 26.0, color);
    } else {
        draw_line(
            pos.x - 12.0,
            pos.y + 11.0,
            pos.x + 11.0,
            pos.y - 12.0,
            5.0,
            color,
        );
        draw_circle(pos.x - 12.0, pos.y + 11.0, 5.0, color);
        draw_rectangle(pos.x + 7.0, pos.y - 15.0, 8.0, 8.0, color);
    }
}

fn draw_hourglass(pos: Vec2, color: Color) {
    draw_line(
        pos.x - 9.0,
        pos.y - 14.0,
        pos.x + 9.0,
        pos.y - 14.0,
        2.0,
        color,
    );
    draw_line(
        pos.x - 9.0,
        pos.y + 14.0,
        pos.x + 9.0,
        pos.y + 14.0,
        2.0,
        color,
    );
    draw_line(
        pos.x - 7.0,
        pos.y - 12.0,
        pos.x + 7.0,
        pos.y + 12.0,
        2.0,
        color,
    );
    draw_line(
        pos.x + 7.0,
        pos.y - 12.0,
        pos.x - 7.0,
        pos.y + 12.0,
        2.0,
        color,
    );
}

fn draw_flag(pos: Vec2) {
    draw_line(
        pos.x - 8.0,
        pos.y - 14.0,
        pos.x - 8.0,
        pos.y + 14.0,
        3.0,
        GOLD,
    );
    draw_triangle(
        vec2(pos.x - 8.0, pos.y - 13.0),
        vec2(pos.x + 12.0, pos.y - 7.0),
        vec2(pos.x - 8.0, pos.y - 1.0),
        Color::new(0.88, 0.62, 0.16, 1.0),
    );
}

fn draw_compass(pos: Vec2) {
    draw_circle(pos.x, pos.y, 30.0, Color::new(0.10, 0.08, 0.05, 1.0));
    draw_circle_lines(pos.x, pos.y, 30.0, 1.0, GOLD_SOFT);
    draw_triangle(
        vec2(pos.x, pos.y - 32.0),
        vec2(pos.x - 7.0, pos.y),
        vec2(pos.x + 7.0, pos.y),
        GOLD,
    );
    draw_triangle(
        vec2(pos.x, pos.y + 32.0),
        vec2(pos.x - 7.0, pos.y),
        vec2(pos.x + 7.0, pos.y),
        MUTED,
    );
    draw_line(pos.x - 26.0, pos.y, pos.x + 26.0, pos.y, 1.0, GOLD_SOFT);
    draw_line(pos.x, pos.y - 26.0, pos.x, pos.y + 26.0, 1.0, GOLD_SOFT);
}

fn draw_heart(pos: Vec2, size: f32, color: Color) {
    draw_circle(pos.x - size * 0.45, pos.y - size * 0.2, size * 0.55, color);
    draw_circle(pos.x + size * 0.45, pos.y - size * 0.2, size * 0.55, color);
    draw_triangle(
        vec2(pos.x - size, pos.y),
        vec2(pos.x + size, pos.y),
        vec2(pos.x, pos.y + size * 1.25),
        color,
    );
}

fn draw_skull(pos: Vec2) {
    draw_circle(pos.x, pos.y, 8.0, INK);
    draw_rectangle(pos.x - 5.0, pos.y + 4.0, 10.0, 8.0, INK);
    draw_circle(pos.x - 3.0, pos.y - 1.0, 2.0, BAR_BG);
    draw_circle(pos.x + 3.0, pos.y - 1.0, 2.0, BAR_BG);
}

fn draw_role_badge(rect: Rect, label: &str, active: bool) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if active {
            Color::new(0.07, 0.22, 0.10, 0.96)
        } else {
            Color::new(0.12, 0.08, 0.07, 0.86)
        },
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, GOLD_SOFT);
    draw_text_centered_in_box(
        label,
        rect.x + 4.0,
        rect.y + 2.0,
        rect.w - 8.0,
        rect.h - 4.0,
        12.0,
        INK,
    );
}

fn draw_drag_hint(pos: Vec2) {
    draw_line(pos.x - 8.0, pos.y, pos.x + 8.0, pos.y, 1.0, GOLD_SOFT);
    draw_line(pos.x, pos.y - 8.0, pos.x, pos.y + 8.0, 1.0, GOLD_SOFT);
    draw_ui_text_ex(
        "Drag to reposition",
        pos.x + 16.0,
        pos.y + 5.0,
        TextStyle::new(12.0, MUTED).params(),
    );
}

fn timer_label(run: &MissionRun) -> (String, Color) {
    let seconds = if let Some(limit) = run.time_limit {
        (limit - run.elapsed).max(0.0)
    } else {
        run.elapsed
    };
    let color = if run.time_limit.is_some() && seconds <= 12.0 {
        Color::new(1.0, 0.45, 0.32, 1.0)
    } else {
        INK
    };
    let minutes = (seconds / 60.0).floor() as i32;
    let seconds = (seconds % 60.0).floor() as i32;
    (format!("{:02}:{:02}", minutes, seconds), color)
}
