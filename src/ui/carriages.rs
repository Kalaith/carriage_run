//! Carriage chassis shop: buy and switch between wagons that trade slots,
//! speed, and hull strength.

use super::upgrade_visuals::{draw_panel, draw_panel_with_fill, GOLD_SOFT, INK, MUTED, PANEL_ALT};
use super::widgets::{draw_menu_backdrop, draw_top_nav, virtual_button};
use super::{UiAction, UiContext};
use crate::data::ChassisDef;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_carriages(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(70.0);
    draw_top_nav(ctx, "Carriages", mouse, actions);

    let banner = Rect::new(82.0, 110.0, 1116.0, 108.0);
    draw_panel(banner, false);
    draw_ui_text_ex(
        "Choose Your Wagon",
        banner.x + 24.0,
        banner.y + 30.0,
        TextStyle::new(24.0, INK).params(),
    );
    // Frame tuning: a mutually-exclusive build-identity choice (opportunity cost).
    let frame_id = &ctx.session.campaign.carriage_frame_id;
    let frames = ctx.data.carriage_frames_ordered();
    let n = frames.len().max(1) as f32;
    let fw = (banner.w - 48.0 - (n - 1.0) * 12.0) / n;
    for (i, frame) in frames.iter().enumerate() {
        let selected = &frame.id == frame_id;
        let rect = Rect::new(
            banner.x + 24.0 + i as f32 * (fw + 12.0),
            banner.y + 52.0,
            fw,
            42.0,
        );
        if virtual_button(
            rect,
            &frame.name,
            true,
            if selected {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            mouse,
        ) {
            actions.push(UiAction::SelectFrame(frame.id.clone()));
        }
    }
    if let Some(active) = frames.iter().find(|f| &f.id == frame_id) {
        draw_ui_text_ex(
            &active.description,
            banner.x + 320.0,
            banner.y + 30.0,
            TextStyle::new(15.0, MUTED).params(),
        );
    }

    let chassis = ctx.data.chassis_ordered();
    let card_w = 356.0;
    let gap = 22.0;
    let total = chassis.len() as f32 * card_w + (chassis.len().saturating_sub(1)) as f32 * gap;
    let start_x = (1280.0 - total) * 0.5;
    for (index, def) in chassis.iter().enumerate() {
        let rect = Rect::new(
            start_x + index as f32 * (card_w + gap),
            238.0,
            card_w,
            398.0,
        );
        draw_chassis_card(ctx, def, rect, mouse, actions);
    }

    if virtual_button(
        Rect::new(84.0, 648.0, 160.0, 42.0),
        "Back to Shop",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenShop);
    }
    if virtual_button(
        Rect::new(262.0, 648.0, 160.0, 42.0),
        "Routes",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::OpenMap);
    }
}

fn draw_chassis_card(
    ctx: &UiContext<'_>,
    def: &ChassisDef,
    rect: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let campaign = &ctx.session.campaign;
    let owned = campaign.is_chassis_owned(&def.id);
    let active = campaign.chassis_id == def.id;

    let fill = if active {
        Color::new(0.07, 0.11, 0.08, 0.98)
    } else {
        PANEL_ALT
    };
    draw_panel_with_fill(rect, fill, owned);

    draw_ui_text_ex(
        &def.name,
        rect.x + 22.0,
        rect.y + 36.0,
        TextStyle::new(24.0, INK).params(),
    );
    if active {
        draw_badge(
            Rect::new(rect.right() - 92.0, rect.y + 16.0, 74.0, 24.0),
            "Active",
            Color::new(0.10, 0.28, 0.14, 1.0),
            Color::new(0.62, 0.94, 0.66, 1.0),
        );
    } else if owned {
        draw_badge(
            Rect::new(rect.right() - 92.0, rect.y + 16.0, 74.0, 24.0),
            "Owned",
            Color::new(0.14, 0.16, 0.18, 1.0),
            MUTED,
        );
    }

    draw_wagon_preview(
        Rect::new(rect.x + 22.0, rect.y + 52.0, rect.w - 44.0, 96.0),
        def.slots,
    );

    let mut y = rect.y + 176.0;
    draw_stat_row(rect, y, "Slots", &def.slots.to_string());
    y += 30.0;
    draw_stat_row(rect, y, "Speed", &speed_label(def.speed_mult));
    y += 30.0;
    draw_stat_row(rect, y, "Hull", &hull_label(def.health_mult));

    draw_text_block(
        &def.description,
        rect.x + 22.0,
        rect.y + 274.0,
        rect.w - 44.0,
        60.0,
        14.0,
        3.0,
        Color::new(0.80, 0.82, 0.72, 1.0),
    );

    let button = Rect::new(rect.x + 22.0, rect.bottom() - 50.0, rect.w - 44.0, 34.0);
    if active {
        virtual_button(button, "Equipped", false, ButtonTone::Muted, mouse);
    } else if owned {
        if virtual_button(button, "Switch To", true, ButtonTone::Positive, mouse) {
            actions.push(UiAction::SelectChassis(def.id.clone()));
        }
    } else {
        let affordable = campaign.gold >= def.cost;
        let label = if affordable {
            format!("Buy - {} Gold", def.cost)
        } else {
            format!("Need {} Gold", def.cost)
        };
        if virtual_button(button, &label, affordable, ButtonTone::Primary, mouse) {
            actions.push(UiAction::BuyChassis(def.id.clone()));
        }
    }
}

fn draw_stat_row(card: Rect, y: f32, label: &str, value: &str) {
    draw_ui_text_ex(
        label,
        card.x + 24.0,
        y,
        TextStyle::new(16.0, MUTED).params(),
    );
    draw_text_right(value, card.right() - 24.0, y, TextStyle::new(16.0, INK));
}

fn speed_label(mult: f32) -> String {
    let delta = ((mult - 1.0) * 100.0).round() as i32;
    match delta.cmp(&0) {
        std::cmp::Ordering::Greater => format!("Fast (+{}%)", delta),
        std::cmp::Ordering::Less => format!("Slow ({}%)", delta),
        std::cmp::Ordering::Equal => "Even".to_owned(),
    }
}

fn hull_label(mult: f32) -> String {
    let delta = ((mult - 1.0) * 100.0).round() as i32;
    match delta.cmp(&0) {
        std::cmp::Ordering::Greater => format!("Tough (+{}%)", delta),
        std::cmp::Ordering::Less => format!("Light ({}%)", delta),
        std::cmp::Ordering::Equal => "Standard".to_owned(),
    }
}

/// A simple schematic wagon whose width and roof grow with the slot count.
fn draw_wagon_preview(rect: Rect, slots: usize) {
    let width = 72.0 + slots as f32 * 22.0;
    let height = 52.0;
    let body = Rect::new(
        rect.center().x - width * 0.5,
        rect.center().y - height * 0.5,
        width,
        height,
    );
    draw_rectangle(
        body.x,
        body.y,
        body.w,
        body.h,
        Color::new(0.52, 0.30, 0.15, 1.0),
    );
    draw_rectangle(
        body.x + 6.0,
        body.y + 8.0,
        body.w - 12.0,
        16.0,
        Color::new(0.74, 0.54, 0.28, 1.0),
    );
    draw_rectangle_lines(
        body.x,
        body.y,
        body.w,
        body.h,
        2.0,
        Color::new(0.16, 0.08, 0.04, 1.0),
    );
    for i in 0..slots {
        let cx = body.x + 14.0 + i as f32 * ((body.w - 28.0) / slots.max(1) as f32) + 6.0;
        draw_circle(cx, body.y + 34.0, 5.0, Color::new(0.92, 0.78, 0.36, 0.85));
    }
    let wheel = Color::new(0.16, 0.09, 0.05, 1.0);
    for wx in [body.x + 12.0, body.right() - 12.0] {
        draw_circle(wx, body.bottom(), 9.0, wheel);
        draw_circle_lines(wx, body.bottom(), 11.0, 2.0, GOLD_SOFT);
    }
}
