//! Shared UI widgets and menu backdrop drawing.

use super::upgrade_visuals::{
    draw_crest, draw_panel_with_fill, draw_stat_icon, draw_upgrade_backdrop, footer_button,
    nav_tile, GOLD as UI_GOLD, GOLD_SOFT, INK, MUTED, PANEL, PANEL_ALT,
};
use super::{UiAction, UiContext, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_top_nav(
    ctx: &UiContext<'_>,
    title: &str,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let rect = Rect::new(18.0, 14.0, LOGICAL_WIDTH - 36.0, 86.0);
    draw_panel_with_fill(rect, Color::new(0.032, 0.042, 0.037, 0.98), false);
    draw_line(
        rect.x + 24.0,
        rect.bottom() - 8.0,
        rect.right() - 24.0,
        rect.bottom() - 8.0,
        1.0,
        GOLD_SOFT,
    );
    draw_crest(Rect::new(rect.x + 20.0, rect.y + 10.0, 70.0, 66.0));
    draw_ui_text_ex(
        title,
        rect.x + 108.0,
        rect.y + 52.0,
        TextStyle::new(32.0, INK).params(),
    );

    draw_header_stat(
        Rect::new(rect.x + 286.0, rect.y + 28.0, 116.0, 42.0),
        "Gold",
        &format!("Gold {}", ctx.session.campaign.gold),
        "gold",
    );
    draw_header_stat(
        Rect::new(rect.x + 418.0, rect.y + 28.0, 136.0, 42.0),
        "Carriage",
        &format!("Carriage L{}", ctx.session.campaign.carriage_level),
        "level",
    );

    if footer_button(
        Rect::new(rect.x + 570.0, rect.y + 34.0, 54.0, 28.0),
        "Save",
        mouse,
    ) {
        actions.push(UiAction::Save);
    }
    if footer_button(
        Rect::new(rect.x + 632.0, rect.y + 34.0, 54.0, 28.0),
        "Load",
        mouse,
    ) && ctx.save_exists
    {
        actions.push(UiAction::Load);
    }
    if footer_button(
        Rect::new(rect.x + 694.0, rect.y + 34.0, 54.0, 28.0),
        "Title",
        mouse,
    ) {
        actions.push(UiAction::ReturnTitle);
    }

    let active = active_nav_id(title);
    let nav = [
        ("Map", "map", UiAction::OpenMap, active == "map"),
        ("Shop", "shop", UiAction::OpenShop, active == "shop"),
        ("Guards", "guards", UiAction::OpenGuards, active == "guards"),
        (
            "Upgrades",
            "up",
            UiAction::OpenUpgrades,
            active == "upgrades",
        ),
        (
            "Settings",
            "settings",
            UiAction::OpenSettings,
            active == "settings",
        ),
    ];
    let mut x = rect.right() - 496.0;
    for (label, icon, action, active) in nav {
        if nav_tile(
            Rect::new(x, rect.y + 6.0, 82.0, 76.0),
            label,
            icon,
            active,
            mouse,
        ) {
            actions.push(action);
        }
        x += 92.0;
    }
}

fn active_nav_id(title: &str) -> &'static str {
    match title {
        "Hire Shop" => "shop",
        "Guard Roster" => "guards",
        "Settings" => "settings",
        "Mission Loadout" | "Routes" => "map",
        _ => "",
    }
}

fn draw_header_stat(rect: Rect, label: &str, value: &str, icon: &str) {
    draw_panel_with_fill(rect, Color::new(0.040, 0.050, 0.045, 0.84), false);
    draw_stat_icon(icon, vec2(rect.x + 20.0, rect.y + 22.0));
    draw_ui_text_ex(
        label,
        rect.x + 42.0,
        rect.y + 17.0,
        TextStyle::new(11.0, UI_GOLD).params(),
    );
    draw_ui_text_ex(
        value,
        rect.x + 42.0,
        rect.y + 34.0,
        TextStyle::new(13.0, INK).params(),
    );
}

pub(super) fn draw_mix_list(rect: Rect, title: &str, values: &[String]) {
    draw_ui_text_ex(
        title,
        rect.x,
        rect.y,
        TextStyle::new(18.0, UI_GOLD).params(),
    );
    let mut y = rect.y + 26.0;
    for value in values {
        draw_badge(
            Rect::new(rect.x, y, rect.w, 20.0),
            &format_label(value),
            Color::new(0.06, 0.08, 0.07, 0.92),
            MUTED,
        );
        y += 24.0;
    }
}

pub(super) fn draw_menu_backdrop(scroll: f32) {
    draw_upgrade_backdrop();
    for i in 0..10 {
        let y = ((i as f32 * 94.0 + scroll) % (LOGICAL_HEIGHT + 70.0)) - 50.0;
        draw_rectangle(895.0, y, 8.0, 42.0, Color::new(0.80, 0.64, 0.34, 0.30));
    }
}

pub(super) fn virtual_button(
    rect: Rect,
    text: &str,
    enabled: bool,
    tone: ButtonTone,
    mouse: Vec2,
) -> bool {
    let style = ButtonStyle::from_tone(tone);
    let hovered = enabled && rect.contains_point(mouse);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let activated = hovered && is_mouse_button_released(MouseButton::Left);
    let fill = if !enabled {
        Color::new(PANEL.r, PANEL.g, PANEL.b, 0.58)
    } else if pressed {
        style.pressed
    } else if hovered {
        match tone {
            ButtonTone::Positive => Color::new(0.08, 0.25, 0.13, 0.98),
            ButtonTone::Primary => Color::new(0.12, 0.16, 0.12, 0.98),
            ButtonTone::Muted => Color::new(0.11, 0.10, 0.09, 0.98),
            _ => Color::new(PANEL_ALT.r + 0.03, PANEL_ALT.g + 0.03, PANEL_ALT.b, 0.98),
        }
    } else if matches!(tone, ButtonTone::Positive) {
        Color::new(0.06, 0.18, 0.10, 0.96)
    } else if matches!(tone, ButtonTone::Primary) {
        Color::new(0.08, 0.12, 0.10, 0.96)
    } else {
        PANEL
    };
    draw_panel_with_fill(rect, fill, hovered || matches!(tone, ButtonTone::Positive));
    if !enabled {
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.0, 0.0, 0.0, 0.28),
        );
    }
    draw_text_centered_in_box_ex(
        text,
        rect.x + 8.0,
        rect.y + if pressed { 2.0 } else { 0.0 },
        rect.w - 16.0,
        rect.h,
        TextStyle::new(
            17.0,
            if enabled {
                if matches!(tone, ButtonTone::Muted) {
                    MUTED
                } else {
                    INK
                }
            } else {
                Color::new(
                    style.text_color.r,
                    style.text_color.g,
                    style.text_color.b,
                    0.42,
                )
            },
        ),
    );
    activated
}

pub(super) fn star_label(stars: u8) -> String {
    match stars {
        0 => "No Stars".to_owned(),
        1 => "*".to_owned(),
        2 => "**".to_owned(),
        _ => "***".to_owned(),
    }
}

pub(super) fn format_label(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
