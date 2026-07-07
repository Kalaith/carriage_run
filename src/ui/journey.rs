//! Expedition (roguelite journey) hub and end-of-run summary.

use super::upgrade_visuals::{draw_panel, draw_section_label, GOLD as UI_GOLD, INK, MUTED};
use super::widgets::{draw_menu_backdrop, virtual_button};
use super::{UiAction, UiContext};
use crate::state::Journey;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_journey(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(64.0);
    let Some(journey) = &ctx.session.journey else {
        // No active expedition (shouldn't normally happen) — offer a way out.
        let panel = Rect::new(420.0, 240.0, 440.0, 160.0);
        draw_panel(panel, true);
        if virtual_button(
            Rect::new(panel.x + 140.0, panel.y + 90.0, 160.0, 44.0),
            "Back to Camp",
            true,
            ButtonTone::Primary,
            mouse,
        ) {
            actions.push(UiAction::OpenMap);
        }
        return;
    };

    if journey.alive {
        draw_hub(journey, mouse, actions);
    } else {
        draw_summary(journey, mouse, actions);
    }
}

fn draw_hub(journey: &Journey, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let panel = Rect::new(360.0, 96.0, 560.0, 520.0);
    draw_panel(panel, true);
    draw_text_centered_in_box(
        &format!("Expedition — Leg {}", journey.leg),
        panel.x + 30.0,
        panel.y + 28.0,
        panel.w - 60.0,
        44.0,
        34.0,
        INK,
    );
    if journey.last_reward > 0 {
        draw_text_centered_in_box(
            &format!(
                "{} cleared: +{} gold banked",
                journey.last_mission_name, journey.last_reward
            ),
            panel.x + 30.0,
            panel.y + 74.0,
            panel.w - 60.0,
            26.0,
            18.0,
            Color::new(0.42, 0.86, 0.46, 1.0),
        );
    } else {
        draw_text_centered_in_box(
            "Load up and set out. Damage carries between legs.",
            panel.x + 30.0,
            panel.y + 74.0,
            panel.w - 60.0,
            26.0,
            18.0,
            MUTED,
        );
    }

    draw_section_label(
        "Convoy Status",
        panel.x + 40.0,
        panel.y + 128.0,
        panel.w - 80.0,
    );
    let health_pct = (journey.carriage_health_ratio * 100.0).round() as i32;
    draw_stat_line(
        panel,
        panel.y + 158.0,
        "Carriage Health",
        &format!("{}%", health_pct),
    );
    draw_stat_line(
        panel,
        panel.y + 194.0,
        "Banked Gold",
        &format!("{}", journey.banked_gold),
    );
    draw_stat_line(
        panel,
        panel.y + 230.0,
        "Next Leg Reward",
        &format!("+{}", Journey::leg_reward(journey.leg)),
    );
    let next_scale = ((journey.difficulty_scale() - 1.0) * 100.0).round() as i32;
    draw_stat_line(
        panel,
        panel.y + 266.0,
        "Threat Level",
        &format!("+{}%", next_scale),
    );

    let mut y = panel.y + 322.0;
    if virtual_button(
        Rect::new(panel.x + 60.0, y, panel.w - 120.0, 46.0),
        &format!("Press On to Leg {}", journey.leg),
        true,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::JourneyPressOn);
    }
    y += 58.0;
    let repair_label = if journey.carriage_health_ratio >= 0.995 {
        "Carriage Fully Repaired".to_owned()
    } else {
        format!("Field Repairs ({} gold)", journey.repair_cost())
    };
    if virtual_button(
        Rect::new(panel.x + 60.0, y, panel.w - 120.0, 46.0),
        &repair_label,
        journey.can_repair(),
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::JourneyRepair);
    }
    y += 58.0;
    if virtual_button(
        Rect::new(panel.x + 60.0, y, panel.w - 120.0, 46.0),
        &format!("Bank {} Gold & Return", journey.banked_gold),
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::JourneyBank);
    }
}

fn draw_summary(journey: &Journey, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let panel = Rect::new(400.0, 150.0, 480.0, 360.0);
    draw_panel(panel, true);
    draw_text_centered_in_box(
        "Expedition Ended",
        panel.x + 30.0,
        panel.y + 34.0,
        panel.w - 60.0,
        46.0,
        36.0,
        Color::new(1.0, 0.62, 0.42, 1.0),
    );
    draw_text_centered_in_box(
        &format!("The convoy fell on leg {}.", journey.leg),
        panel.x + 30.0,
        panel.y + 88.0,
        panel.w - 60.0,
        28.0,
        20.0,
        MUTED,
    );

    draw_stat_line(
        panel,
        panel.y + 150.0,
        "Legs Cleared",
        &format!("{}", journey.leg.saturating_sub(1)),
    );
    draw_stat_line(
        panel,
        panel.y + 186.0,
        "Banked",
        &format!("{}", journey.banked_gold),
    );
    draw_stat_line(
        panel,
        panel.y + 222.0,
        "Salvaged (half)",
        &format!("+{} gold", journey.payout),
    );

    if virtual_button(
        Rect::new(panel.x + 150.0, panel.bottom() - 70.0, 180.0, 46.0),
        "Return to Camp",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::JourneyBank);
    }
}

fn draw_stat_line(panel: Rect, y: f32, label: &str, value: &str) {
    draw_ui_text_ex(
        label,
        panel.x + 60.0,
        y,
        TextStyle::new(19.0, MUTED).params(),
    );
    draw_text_right(
        value,
        panel.right() - 60.0,
        y,
        TextStyle::new(19.0, UI_GOLD),
    );
}
