//! Rich carriage upgrade screen.

use super::upgrade_visuals::{
    draw_crest, draw_equipment_icon, draw_panel, draw_panel_with_fill, draw_section_label,
    draw_section_title, draw_stat_icon, draw_upgrade_backdrop, draw_upgrade_icon, footer_button,
    gold_button, nav_tile, small_close, GOLD as UI_GOLD, GOLD_SOFT, INK, MUTED, PANEL_ALT,
};
use super::widgets::virtual_button;
use super::{UiAction, UiContext};
use crate::data::UpgradeDef;
use crate::state::CarriageEquipment;
use macroquad::prelude::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, vec2, Color, Rect, Vec2,
};
use macroquad_toolkit::prelude::{
    draw_badge, draw_text_block, draw_text_centered_in_box, ButtonTone, RectExt, TextStyle,
};
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_upgrades(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_upgrade_backdrop();
    draw_header(ctx, mouse, actions);
    draw_section_title("Available Upgrades", 142.0);
    draw_available_upgrades(ctx, mouse, actions);
    draw_loadout_and_owned(ctx, mouse, actions);
    draw_footer(ctx, mouse, actions);
}

fn draw_header(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_line(
        42.0,
        116.0,
        1238.0,
        116.0,
        2.0,
        Color::new(0.56, 0.38, 0.16, 0.62),
    );
    draw_crest(Rect::new(42.0, 20.0, 96.0, 92.0));
    draw_ui_text_ex("Carriage", 166.0, 57.0, TextStyle::new(42.0, INK).params());
    draw_ui_text_ex("Upgrades", 166.0, 96.0, TextStyle::new(42.0, INK).params());
    draw_stats_panel(ctx, Rect::new(360.0, 26.0, 374.0, 72.0));
    draw_nav_tiles(ctx, mouse, actions);
}

fn draw_stats_panel(ctx: &UiContext<'_>, rect: Rect) {
    draw_panel(rect, false);
    let campaign = &ctx.session.campaign;
    draw_stat(
        Rect::new(rect.x + 12.0, rect.y + 10.0, 104.0, 52.0),
        "Gold",
        &campaign.gold.to_string(),
        "gold",
    );
    draw_stat(
        Rect::new(rect.x + 130.0, rect.y + 10.0, 112.0, 52.0),
        "Carriage",
        &format!("Level {}", campaign.carriage_level),
        "level",
    );
    draw_stat(
        Rect::new(rect.x + 256.0, rect.y + 10.0, 104.0, 52.0),
        "Slots",
        &format!("{}/{}", campaign.carriage_equipment_slot_count(), 4),
        "slots",
    );
}

fn draw_stat(rect: Rect, label: &str, value: &str, icon: &str) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.035, 0.042, 0.038, 0.76),
    );
    draw_stat_icon(icon, vec2(rect.x + 22.0, rect.y + 28.0));
    draw_ui_text_ex(
        label,
        rect.x + 44.0,
        rect.y + 18.0,
        TextStyle::new(12.0, UI_GOLD).params(),
    );
    draw_ui_text_ex(
        value,
        rect.x + 44.0,
        rect.y + 43.0,
        TextStyle::new(19.0, INK).params(),
    );
}

fn draw_nav_tiles(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let nav = [
        ("Map", "map", UiAction::OpenMap, false),
        ("Shop", "shop", UiAction::OpenShop, false),
        ("Guards", "guards", UiAction::OpenGuards, false),
        ("Upgrades", "up", UiAction::OpenUpgrades, true),
        ("Settings", "settings", UiAction::OpenSettings, false),
    ];
    let mut x = 766.0;
    for (label, icon, action, active) in nav {
        if nav_tile(Rect::new(x, 22.0, 82.0, 76.0), label, icon, active, mouse) {
            actions.push(action);
        }
        x += 92.0;
    }
    let _ = ctx.loaded_assets;
}

fn draw_available_upgrades(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let upgrades = ordered_upgrades(ctx);
    let count = upgrades.len().max(1) as f32;
    let start_x = 44.0;
    let gap = 12.0;
    let usable = 1280.0 - start_x * 2.0;
    let card_w = (usable - gap * (count - 1.0)) / count;
    for (index, upgrade) in upgrades.iter().enumerate() {
        let rect = Rect::new(
            start_x + index as f32 * (card_w + gap),
            176.0,
            card_w,
            236.0,
        );
        draw_upgrade_card(ctx, upgrade, rect, mouse, actions);
    }
}

fn ordered_upgrades<'a>(ctx: &'a UiContext<'_>) -> Vec<&'a UpgradeDef> {
    let order = [
        "carriage_armor",
        "reinforced_wheels",
        "cargo_straps",
        "spiked_hubs",
        "warding_lantern",
        "mounted_archer",
        "guard_training",
        "repair_kit",
    ];
    order
        .iter()
        .filter_map(|id| ctx.data.upgrades.get(id))
        .collect()
}

fn draw_upgrade_card(
    ctx: &UiContext<'_>,
    upgrade: &UpgradeDef,
    rect: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let campaign = &ctx.session.campaign;
    let level = campaign.upgrade_level(&upgrade.id);
    let cost = campaign.upgrade_cost(upgrade);
    let can_buy = cost.is_some_and(|cost| campaign.gold >= cost);
    let hovered = rect.contains_point(mouse);
    draw_panel_with_fill(
        rect,
        if hovered {
            Color::new(0.075, 0.085, 0.066, 0.98)
        } else {
            PANEL_ALT
        },
        cost.is_some(),
    );
    draw_circle(
        rect.x + rect.w * 0.5,
        rect.y + 58.0,
        39.0,
        Color::new(0.018, 0.020, 0.018, 0.92),
    );
    draw_circle_lines(rect.x + rect.w * 0.5, rect.y + 58.0, 40.0, 2.0, GOLD_SOFT);
    draw_upgrade_icon(&upgrade.id, vec2(rect.x + rect.w * 0.5, rect.y + 58.0), 1.0);
    draw_text_centered_in_box(
        &upgrade.name,
        rect.x + 14.0,
        rect.y + 107.0,
        rect.w - 28.0,
        34.0,
        15.0,
        INK,
    );
    draw_text_centered_in_box(
        &format!("Level {} / {}", level, upgrade.max_level),
        rect.x + 18.0,
        rect.y + 137.0,
        rect.w - 36.0,
        20.0,
        13.0,
        Color::new(0.98, 0.78, 0.42, 1.0),
    );
    draw_text_block(
        &upgrade.description,
        rect.x + 18.0,
        rect.y + 162.0,
        rect.w - 36.0,
        42.0,
        12.0,
        3.0,
        Color::new(0.80, 0.82, 0.72, 1.0),
    );

    let label = cost
        .map(|cost| {
            if can_buy {
                format!("Buy {}", cost)
            } else {
                format!("Need {}", cost)
            }
        })
        .unwrap_or_else(|| "Max Level".to_owned());
    if gold_button(
        Rect::new(rect.x + 16.0, rect.bottom() - 42.0, rect.w - 32.0, 30.0),
        &label,
        can_buy,
        mouse,
    ) {
        actions.push(UiAction::BuyUpgrade(upgrade.id.clone()));
    }
}

fn draw_loadout_and_owned(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let panel = Rect::new(78.0, 436.0, 1124.0, 202.0);
    draw_panel(panel, false);
    draw_line(
        panel.x + 450.0,
        panel.y + 14.0,
        panel.x + 450.0,
        panel.bottom() - 14.0,
        1.0,
        GOLD_SOFT,
    );
    draw_section_label("Current Loadout", panel.x + 42.0, panel.y + 24.0, 344.0);
    draw_equipped_slots(
        ctx,
        mouse,
        actions,
        Rect::new(panel.x + 34.0, panel.y + 58.0, 386.0, 128.0),
    );
    draw_section_label("Owned Upgrades", panel.x + 494.0, panel.y + 24.0, 560.0);
    draw_owned_upgrades(
        ctx,
        mouse,
        actions,
        Rect::new(panel.x + 482.0, panel.y + 56.0, 618.0, 132.0),
    );
}

fn draw_equipped_slots(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>, rect: Rect) {
    let slot_count = ctx.session.campaign.carriage_equipment_slot_count();
    let card_w = if slot_count <= 2 { 174.0 } else { 86.0 };
    let gap = if slot_count <= 2 { 16.0 } else { 10.0 };
    for slot in 0..slot_count {
        let card = Rect::new(
            rect.x + slot as f32 * (card_w + gap),
            rect.y,
            card_w,
            rect.h,
        );
        let equipment = ctx
            .session
            .campaign
            .selected_equipment_ids
            .get(slot)
            .map(|id| CarriageEquipment::from_id(id));
        draw_slot_card(ctx, card, slot, equipment, mouse, actions);
    }
}

fn draw_slot_card(
    ctx: &UiContext<'_>,
    rect: Rect,
    slot: usize,
    equipment: Option<CarriageEquipment>,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_panel_with_fill(
        rect,
        if equipment.is_some() {
            Color::new(0.07, 0.10, 0.09, 0.98)
        } else {
            Color::new(0.055, 0.052, 0.046, 0.90)
        },
        equipment.is_some(),
    );
    draw_text_centered_in_box(
        &format!("Slot {}", slot + 1),
        rect.x + 8.0,
        rect.y + 12.0,
        rect.w - 16.0,
        18.0,
        13.0,
        UI_GOLD,
    );
    if let Some(equipment) = equipment {
        draw_circle(
            rect.x + rect.w * 0.5,
            rect.y + 55.0,
            31.0,
            Color::new(0.015, 0.017, 0.014, 0.92),
        );
        draw_circle_lines(rect.x + rect.w * 0.5, rect.y + 55.0, 32.0, 2.0, GOLD_SOFT);
        draw_equipment_icon(equipment, vec2(rect.x + rect.w * 0.5, rect.y + 55.0), 0.78);
        draw_text_centered_in_box(
            equipment.label(),
            rect.x + 8.0,
            rect.y + 88.0,
            rect.w - 16.0,
            24.0,
            14.0,
            INK,
        );
        draw_text_centered_in_box(
            &format!("Level {}", ctx.session.campaign.equipment_level(equipment)),
            rect.x + 8.0,
            rect.bottom() - 26.0,
            rect.w - 16.0,
            16.0,
            12.0,
            UI_GOLD,
        );
        if small_close(
            Rect::new(rect.right() - 28.0, rect.y + 10.0, 20.0, 20.0),
            mouse,
        ) {
            actions.push(UiAction::ClearEquipmentSlot(slot));
        }
    } else {
        draw_circle_lines(
            rect.x + rect.w * 0.5,
            rect.y + 58.0,
            24.0,
            2.0,
            Color::new(0.72, 0.58, 0.30, 0.45),
        );
        draw_line(
            rect.x + rect.w * 0.5 - 12.0,
            rect.y + 58.0,
            rect.x + rect.w * 0.5 + 12.0,
            rect.y + 58.0,
            4.0,
            GOLD_SOFT,
        );
        draw_line(
            rect.x + rect.w * 0.5,
            rect.y + 46.0,
            rect.x + rect.w * 0.5,
            rect.y + 70.0,
            4.0,
            GOLD_SOFT,
        );
        draw_text_centered_in_box(
            "Empty Slot",
            rect.x + 8.0,
            rect.y + 88.0,
            rect.w - 16.0,
            22.0,
            14.0,
            INK,
        );
    }
}

fn draw_owned_upgrades(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>, rect: Rect) {
    let upgrades = ordered_upgrades(ctx);
    let owned = upgrades
        .into_iter()
        .filter(|upgrade| {
            let level = ctx.session.campaign.upgrade_level(&upgrade.id);
            if level == 0 {
                return false;
            }
            if let Some(equipment) = equipment_for_upgrade(&upgrade.id) {
                !ctx.session.campaign.is_equipment_equipped(equipment)
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    if owned.is_empty() {
        draw_text_centered_in_box(
            "No owned upgrades waiting to equip.",
            rect.x,
            rect.y + 44.0,
            rect.w,
            30.0,
            18.0,
            MUTED,
        );
        return;
    }

    let card_w = 98.0;
    for (index, upgrade) in owned.iter().take(6).enumerate() {
        let card = Rect::new(
            rect.x + index as f32 * (card_w + 10.0),
            rect.y,
            card_w,
            rect.h,
        );
        draw_owned_card(ctx, upgrade, card, mouse, actions);
    }
}

fn draw_owned_card(
    ctx: &UiContext<'_>,
    upgrade: &UpgradeDef,
    rect: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let equipment = equipment_for_upgrade(&upgrade.id);
    draw_panel_with_fill(rect, PANEL_ALT, true);
    draw_circle(
        rect.x + rect.w * 0.5,
        rect.y + 34.0,
        24.0,
        Color::new(0.015, 0.017, 0.014, 0.92),
    );
    draw_circle_lines(rect.x + rect.w * 0.5, rect.y + 34.0, 25.0, 2.0, GOLD_SOFT);
    draw_upgrade_icon(
        &upgrade.id,
        vec2(rect.x + rect.w * 0.5, rect.y + 34.0),
        0.62,
    );
    draw_text_centered_in_box(
        &upgrade.name,
        rect.x + 7.0,
        rect.y + 60.0,
        rect.w - 14.0,
        34.0,
        11.0,
        INK,
    );
    draw_text_centered_in_box(
        &format!("Level {}", ctx.session.campaign.upgrade_level(&upgrade.id)),
        rect.x + 8.0,
        rect.y + 92.0,
        rect.w - 16.0,
        16.0,
        11.0,
        UI_GOLD,
    );

    if let Some(equipment) = equipment {
        let slot = first_open_equipment_slot(ctx);
        let enabled = slot.is_some();
        if virtual_button(
            Rect::new(rect.x + 10.0, rect.bottom() - 28.0, rect.w - 20.0, 22.0),
            if enabled { "Equip" } else { "Full" },
            enabled,
            ButtonTone::Positive,
            mouse,
        ) {
            if let Some(slot) = slot {
                actions.push(UiAction::AssignEquipmentSlot(
                    slot,
                    equipment.id().to_owned(),
                ));
            }
        }
    } else {
        draw_badge(
            Rect::new(rect.x + 10.0, rect.bottom() - 28.0, rect.w - 20.0, 22.0),
            "Passive",
            Color::new(0.12, 0.14, 0.12, 1.0),
            MUTED,
        );
    }
}

fn first_open_equipment_slot(ctx: &UiContext<'_>) -> Option<usize> {
    let count = ctx.session.campaign.carriage_equipment_slot_count();
    (0..count).find(|slot| {
        ctx.session
            .campaign
            .selected_equipment_ids
            .get(*slot)
            .is_none()
    })
}

fn draw_footer(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    if footer_button(Rect::new(42.0, 666.0, 130.0, 36.0), "Save Game", mouse) {
        actions.push(UiAction::Save);
    }
    if footer_button(Rect::new(184.0, 666.0, 130.0, 36.0), "Load Game", mouse) {
        actions.push(UiAction::Load);
    }
    draw_panel(Rect::new(430.0, 664.0, 420.0, 40.0), false);
    draw_text_centered_in_box(
        "Upgrade your caravan. Survive the road ahead.",
        446.0,
        673.0,
        388.0,
        22.0,
        17.0,
        INK,
    );
    if footer_button(Rect::new(1024.0, 666.0, 184.0, 36.0), "Back To Camp", mouse) {
        actions.push(UiAction::OpenMap);
    }
    let _ = ctx.save_exists;
}

fn equipment_for_upgrade(id: &str) -> Option<CarriageEquipment> {
    match id {
        "carriage_armor" => Some(CarriageEquipment::IronPlating),
        "reinforced_wheels" => Some(CarriageEquipment::ReinforcedWheels),
        "cargo_straps" => Some(CarriageEquipment::CargoStraps),
        "repair_kit" => Some(CarriageEquipment::RepairKit),
        "spiked_hubs" => Some(CarriageEquipment::SpikedHubs),
        "warding_lantern" => Some(CarriageEquipment::WardingLantern),
        _ => None,
    }
}
