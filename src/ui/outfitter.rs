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
    let panel = Rect::new(300.0, 40.0, 680.0, 660.0);
    draw_panel(panel, true);

    draw_text_centered_in_box(
        "Expedition Outfitter",
        panel.x + 30.0,
        panel.y + 22.0,
        panel.w - 60.0,
        44.0,
        34.0,
        INK,
    );
    draw_text_centered_in_box(
        &format!(
            "Expedition Tokens: {}   ·   Gold: {}",
            campaign.expedition_tokens, campaign.gold
        ),
        panel.x + 30.0,
        panel.y + 68.0,
        panel.w - 60.0,
        26.0,
        20.0,
        UI_GOLD,
    );
    if virtual_button(
        Rect::new(panel.right() - 160.0, panel.y + 20.0, 130.0, 34.0),
        "Records",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenRecords);
    }

    // Entry stake selector (gold-in, multiplier-out).
    draw_section_label(
        "Entry Stake",
        panel.x + 36.0,
        panel.y + 106.0,
        panel.w - 72.0,
    );
    let stakes = ctx.data.stakes_ordered();
    let stake_w = (panel.w - 72.0 - 24.0) / 3.0;
    for (i, stake) in stakes.iter().take(3).enumerate() {
        let selected = campaign.selected_stake_id == stake.id;
        let label = if stake.cost > 0 {
            format!("{} ({}g)", stake.name, stake.cost)
        } else {
            stake.name.clone()
        };
        if virtual_button(
            Rect::new(
                panel.x + 36.0 + i as f32 * (stake_w + 12.0),
                panel.y + 132.0,
                stake_w,
                38.0,
            ),
            &label,
            true,
            if selected {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            mouse,
        ) {
            actions.push(UiAction::SelectStake(stake.id.clone()));
        }
    }
    let stake_desc = stakes
        .iter()
        .find(|s| s.id == campaign.selected_stake_id)
        .map(|s| s.description.as_str())
        .unwrap_or("");
    draw_text_centered_in_box(
        stake_desc,
        panel.x + 36.0,
        panel.y + 178.0,
        panel.w - 72.0,
        22.0,
        16.0,
        MUTED,
    );

    draw_section_label(
        "Starting Relics",
        panel.x + 36.0,
        panel.y + 214.0,
        panel.w - 72.0,
    );

    let mut y = panel.y + 244.0;
    for relic in ctx.data.relics_ordered() {
        let owned = campaign.expedition_unlocks.iter().any(|id| id == &relic.id);
        let row = Rect::new(panel.x + 36.0, y, panel.w - 72.0, 52.0);
        draw_panel(row, false);
        draw_ui_text_ex(
            &relic.name,
            row.x + 20.0,
            row.y + 22.0,
            TextStyle::new(20.0, if owned { UI_GOLD } else { INK }).params(),
        );
        draw_ui_text_ex(
            &relic.description,
            row.x + 20.0,
            row.y + 43.0,
            TextStyle::new(15.0, MUTED).params(),
        );
        if owned {
            draw_badge(
                Rect::new(row.right() - 138.0, row.y + 14.0, 116.0, 24.0),
                "Unlocked",
                Color::new(0.12, 0.22, 0.14, 1.0),
                Color::new(0.42, 0.86, 0.46, 1.0),
            );
        } else {
            let affordable = campaign.expedition_tokens >= Journey::STARTING_RELIC_COST;
            if virtual_button(
                Rect::new(row.right() - 148.0, row.y + 8.0, 128.0, 36.0),
                &format!("Unlock ({})", Journey::STARTING_RELIC_COST),
                affordable,
                ButtonTone::Positive,
                mouse,
            ) {
                actions.push(UiAction::UnlockStartingRelic(relic.id.clone()));
            }
        }
        y += 60.0;
    }

    let btn_y = panel.bottom() - 58.0;
    if virtual_button(
        Rect::new(panel.x + 40.0, btn_y, 150.0, 44.0),
        "Back",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenLoadout);
    }
    if virtual_button(
        Rect::new(panel.x + 210.0, btn_y, 190.0, 44.0),
        "Daily Run",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::StartDailyExpedition);
    }
    if virtual_button(
        Rect::new(panel.right() - 250.0, btn_y, 210.0, 44.0),
        "Begin Expedition",
        true,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::StartExpedition);
    }
}
