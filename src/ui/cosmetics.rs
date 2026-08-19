//! Paintshop for cosmetic convoy progression.

use super::upgrade_visuals::{
    draw_panel, draw_panel_with_fill, draw_section_label, GOLD, INK, MUTED, PANEL_ALT,
};
use super::widgets::{draw_menu_backdrop, draw_top_nav, virtual_button};
use super::{UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_cosmetics(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(86.0);
    draw_top_nav(ctx, "Paintshop", mouse, actions);
    let panel = Rect::new(84.0, 122.0, 1112.0, 536.0);
    draw_panel(panel, true);
    draw_section_label(
        "Convoy Colors",
        panel.x + 28.0,
        panel.y + 28.0,
        panel.w - 56.0,
    );
    draw_ui_text_ex(
        &format!(
            "Gold {}  ·  unlock colors, then preview them on the road",
            ctx.session.campaign.gold
        ),
        panel.x + 30.0,
        panel.y + 58.0,
        TextStyle::new(16.0, MUTED).params(),
    );

    let mut livery = Vec::new();
    let mut guards = Vec::new();
    for cosmetic in ctx.data.cosmetics_ordered() {
        if cosmetic.kind == "livery" {
            livery.push(cosmetic);
        } else if cosmetic.kind == "guard_color" {
            guards.push(cosmetic);
        }
    }
    draw_group(
        ctx,
        "Liveries",
        &livery,
        panel.x + 28.0,
        panel.y + 92.0,
        mouse,
        actions,
    );
    draw_group(
        ctx,
        "Guard Sashes",
        &guards,
        panel.x + 28.0,
        panel.y + 310.0,
        mouse,
        actions,
    );
    if virtual_button(
        Rect::new(panel.right() - 180.0, panel.bottom() - 48.0, 148.0, 34.0),
        "Back to Camp",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::OpenMap);
    }
}

fn draw_group(
    ctx: &UiContext<'_>,
    title: &str,
    cosmetics: &[&crate::data::CosmeticDef],
    x: f32,
    y: f32,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_section_label(title, x, y, 1050.0);
    for (index, cosmetic) in cosmetics.iter().enumerate() {
        let card = Rect::new(x + index as f32 * 348.0, y + 26.0, 326.0, 166.0);
        let owned = if cosmetic.kind == "livery" {
            ctx.session.campaign.is_livery_owned(&cosmetic.id)
        } else {
            ctx.session.campaign.is_guard_color_owned(&cosmetic.id)
        };
        let active = (cosmetic.kind == "livery" && ctx.session.campaign.livery_id == cosmetic.id)
            || (cosmetic.kind == "guard_color"
                && ctx.session.campaign.guard_color_id == cosmetic.id);
        draw_panel_with_fill(
            card,
            if active {
                Color::new(0.08, 0.15, 0.11, 0.98)
            } else {
                PANEL_ALT
            },
            active,
        );
        draw_rectangle(
            card.x + 18.0,
            card.y + 18.0,
            46.0,
            46.0,
            tint(cosmetic.tint),
        );
        draw_rectangle_lines(card.x + 18.0, card.y + 18.0, 46.0, 46.0, 2.0, GOLD);
        draw_ui_text_ex(
            &cosmetic.name,
            card.x + 78.0,
            card.y + 38.0,
            TextStyle::new(20.0, INK).params(),
        );
        draw_text_block(
            &cosmetic.description,
            card.x + 18.0,
            card.y + 84.0,
            card.w - 36.0,
            38.0,
            14.0,
            2.0,
            MUTED,
        );
        let label = if active {
            "Equipped".to_owned()
        } else if owned {
            "Equip".to_owned()
        } else {
            format!("Buy {}g", cosmetic.cost)
        };
        let enabled = !active && (owned || ctx.session.campaign.gold >= cosmetic.cost);
        if virtual_button(
            Rect::new(card.right() - 122.0, card.bottom() - 38.0, 104.0, 30.0),
            &label,
            enabled,
            ButtonTone::Positive,
            mouse,
        ) {
            actions.push(if owned {
                UiAction::SelectCosmetic(cosmetic.id.clone())
            } else {
                UiAction::BuyCosmetic(cosmetic.id.clone())
            });
        }
    }
}

fn tint(rgb: [f32; 3]) -> Color {
    Color::new(rgb[0], rgb[1], rgb[2], 1.0)
}
