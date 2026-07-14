//! Expedition Outfitter: the pre-run meta-progression screen. Spend persistent
//! expedition tokens to permanently unlock starting relics, then set out.

use super::upgrade_visuals::{draw_panel, draw_section_label, GOLD as UI_GOLD, INK, MUTED};
use super::widgets::{draw_menu_backdrop, virtual_button};
use super::{UiAction, UiContext};
use crate::state::Journey;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_outfitter(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(64.0);
    let campaign = &ctx.session.campaign;
    let panel = Rect::new(300.0, 56.0, 680.0, 620.0);
    draw_panel(panel, true);

    draw_text_centered_in_box(
        "Expedition Outfitter",
        panel.x + 30.0,
        panel.y + 26.0,
        panel.w - 60.0,
        44.0,
        34.0,
        INK,
    );
    draw_text_centered_in_box(
        "Spend expedition tokens on permanent starting relics.",
        panel.x + 30.0,
        panel.y + 72.0,
        panel.w - 60.0,
        24.0,
        18.0,
        MUTED,
    );
    draw_text_centered_in_box(
        &format!("Expedition Tokens: {}", campaign.expedition_tokens),
        panel.x + 30.0,
        panel.y + 104.0,
        panel.w - 60.0,
        28.0,
        22.0,
        UI_GOLD,
    );

    draw_section_label(
        "Starting Relics",
        panel.x + 36.0,
        panel.y + 146.0,
        panel.w - 72.0,
    );

    let mut y = panel.y + 178.0;
    for relic in ctx.data.relics_ordered() {
        let owned = campaign.expedition_unlocks.iter().any(|id| id == &relic.id);
        let row = Rect::new(panel.x + 36.0, y, panel.w - 72.0, 58.0);
        draw_panel(row, false);
        draw_ui_text_ex(
            &relic.name,
            row.x + 20.0,
            row.y + 24.0,
            TextStyle::new(21.0, if owned { UI_GOLD } else { INK }).params(),
        );
        draw_ui_text_ex(
            &relic.description,
            row.x + 20.0,
            row.y + 46.0,
            TextStyle::new(16.0, MUTED).params(),
        );
        if owned {
            draw_badge(
                Rect::new(row.right() - 140.0, row.y + 16.0, 118.0, 26.0),
                "Unlocked",
                Color::new(0.12, 0.22, 0.14, 1.0),
                Color::new(0.42, 0.86, 0.46, 1.0),
            );
        } else {
            let affordable = campaign.expedition_tokens >= Journey::STARTING_RELIC_COST;
            if virtual_button(
                Rect::new(row.right() - 150.0, row.y + 10.0, 130.0, 38.0),
                &format!("Unlock ({})", Journey::STARTING_RELIC_COST),
                affordable,
                ButtonTone::Positive,
                mouse,
            ) {
                actions.push(UiAction::UnlockStartingRelic(relic.id.clone()));
            }
        }
        y += 66.0;
    }

    if virtual_button(
        Rect::new(panel.x + 60.0, panel.bottom() - 58.0, 200.0, 44.0),
        "Back",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenLoadout);
    }
    if virtual_button(
        Rect::new(panel.right() - 260.0, panel.bottom() - 58.0, 200.0, 44.0),
        "Begin Expedition",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::StartExpedition);
    }
}
