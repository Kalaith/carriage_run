//! Mission loadout screen.

use super::upgrade_visuals::{
    draw_panel, draw_panel_with_fill, draw_section_label, GOLD as UI_GOLD, INK, MUTED, PANEL_ALT,
};
use super::widgets::{
    draw_menu_backdrop, draw_mix_list, draw_top_nav, format_label, virtual_button,
};
use super::{UiAction, UiContext};
use crate::state::GuardKind;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_loadout(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(54.0);
    draw_top_nav(ctx, "Mission Loadout", mouse, actions);

    let mission = ctx
        .data
        .missions
        .get(&ctx.session.campaign.selected_mission_id)
        .or_else(|| ctx.data.missions_ordered().first().copied());
    let Some(mission) = mission else {
        return;
    };

    draw_mission_summary(ctx, mission, Rect::new(80.0, 112.0, 1120.0, 112.0));
    draw_slot_row(ctx, "Melee Guard Slots", false, 246.0, mouse, actions);
    draw_pool(ctx, false, 330.0, mouse, actions);
    draw_slot_row(ctx, "Carriage Ranged Slots", true, 458.0, mouse, actions);
    draw_pool(ctx, true, 542.0, mouse, actions);

    if virtual_button(
        Rect::new(700.0, 642.0, 154.0, 42.0),
        "Back",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenMap);
    }
    if virtual_button(
        Rect::new(872.0, 642.0, 154.0, 42.0),
        "Expedition",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::StartExpedition);
    }
    if virtual_button(
        Rect::new(1044.0, 642.0, 154.0, 42.0),
        "Start",
        ctx.session.campaign.is_mission_unlocked(mission),
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::BeginMission);
    }
}

fn draw_mission_summary(ctx: &UiContext<'_>, mission: &crate::data::MissionDef, rect: Rect) {
    let route_choice = ctx.session.campaign.selected_route_choice(mission);
    let route_name = route_choice
        .map(|choice| choice.name.as_str())
        .unwrap_or(mission.route.as_str());
    let (enemy_mix, hazard_mix) = loadout_mixes(ctx, mission);

    draw_panel(rect, false);
    draw_section_label("Mission Brief", rect.x + 22.0, rect.y + 18.0, 330.0);
    draw_ui_text_ex(
        &mission.name,
        rect.x + 22.0,
        rect.y + 45.0,
        TextStyle::new(26.0, INK).params(),
    );
    draw_badge(
        Rect::new(rect.x + 22.0, rect.y + 50.0, 134.0, 25.0),
        &format_label(&mission.mission_type),
        Color::new(0.12, 0.16, 0.16, 1.0),
        MUTED,
    );
    draw_badge(
        Rect::new(rect.x + 22.0, rect.y + 79.0, 134.0, 25.0),
        route_name,
        Color::new(0.12, 0.16, 0.16, 1.0),
        MUTED,
    );
    draw_text_block(
        &mission.objective,
        rect.x + 176.0,
        rect.y + 28.0,
        420.0,
        54.0,
        16.0,
        3.0,
        MUTED,
    );
    draw_mix_list(
        Rect::new(rect.x + 628.0, rect.y + 24.0, 204.0, 74.0),
        "Threats",
        &enemy_mix,
    );
    draw_mix_list(
        Rect::new(rect.x + 856.0, rect.y + 24.0, 188.0, 74.0),
        "Hazards",
        &hazard_mix,
    );

    if ctx.session.campaign.repair_level > 0 {
        draw_badge(
            Rect::new(rect.right() - 118.0, rect.bottom() - 34.0, 92.0, 25.0),
            &format!("Repair L{}", ctx.session.campaign.repair_level),
            Color::new(0.13, 0.17, 0.17, 1.0),
            INK,
        );
    }
    if let Some(limit) = mission.time_limit.map(|limit| {
        route_choice
            .map(|choice| limit + choice.time_limit_delta)
            .unwrap_or(limit)
            .max(30.0)
    }) {
        draw_badge(
            Rect::new(rect.right() - 222.0, rect.bottom() - 34.0, 92.0, 25.0),
            &format!("{:.0}s", limit),
            Color::new(0.20, 0.12, 0.10, 1.0),
            INK,
        );
    }
}

fn loadout_mixes(
    ctx: &UiContext<'_>,
    mission: &crate::data::MissionDef,
) -> (Vec<String>, Vec<String>) {
    let mut enemies = mission.enemy_mix.clone();
    let mut hazards = mission.hazard_mix.clone();
    if let Some(choice) = ctx.session.campaign.selected_route_choice(mission) {
        enemies.extend(choice.enemy_add.iter().cloned());
        hazards.extend(choice.hazard_add.iter().cloned());
    }
    (compact_mix(enemies), compact_mix(hazards))
}

fn compact_mix(values: Vec<String>) -> Vec<String> {
    if values.len() <= 3 {
        return values;
    }
    let hidden = values.len() - 3;
    let mut compact = values.into_iter().take(3).collect::<Vec<_>>();
    compact.push(format!("{} more", hidden));
    compact
}

fn draw_slot_row(
    ctx: &UiContext<'_>,
    title: &str,
    ranged: bool,
    y: f32,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_section_label(title, 84.0, y - 12.0, 360.0);
    let count = visible_slot_count(ctx, ranged);
    let selected = if ranged {
        &ctx.session.campaign.selected_ranged_ids
    } else {
        &ctx.session.campaign.selected_guard_ids
    };

    for slot in 0..count {
        let rect = Rect::new(84.0 + slot as f32 * 216.0, y, 198.0, 58.0);
        let kind = selected.get(slot).map(|id| GuardKind::from_id(id));
        draw_slot_card(ctx, rect, slot, kind, ranged, mouse, actions);
    }
}

fn draw_slot_card(
    ctx: &UiContext<'_>,
    rect: Rect,
    slot: usize,
    kind: Option<GuardKind>,
    ranged: bool,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_panel_with_fill(
        rect,
        if kind.is_some() {
            PANEL_ALT
        } else {
            Color::new(0.055, 0.052, 0.046, 0.90)
        },
        kind.is_some(),
    );
    draw_ui_text_ex(
        &format!("Slot {}", slot + 1),
        rect.x + 14.0,
        rect.y + 24.0,
        TextStyle::new(16.0, MUTED).params(),
    );
    let label = kind.map(GuardKind::label).unwrap_or("Empty");
    draw_ui_text_ex(
        label,
        rect.x + 14.0,
        rect.y + 47.0,
        TextStyle::new(18.0, INK).params(),
    );
    if let Some(kind) = kind {
        let recovery = ctx.session.campaign.guard_recovery_missions(kind);
        if recovery > 0 {
            draw_badge(
                Rect::new(rect.right() - 86.0, rect.y + 8.0, 70.0, 23.0),
                &format!("Rest {}", recovery),
                Color::new(0.22, 0.13, 0.10, 1.0),
                Color::new(0.96, 0.78, 0.54, 1.0),
            );
        }
    }
    if virtual_button(
        Rect::new(rect.right() - 68.0, rect.bottom() - 31.0, 52.0, 24.0),
        "Clear",
        kind.is_some(),
        ButtonTone::Muted,
        mouse,
    ) {
        actions.push(if ranged {
            UiAction::ClearRangedSlot(slot)
        } else {
            UiAction::ClearGuardSlot(slot)
        });
    }
}

fn draw_pool(ctx: &UiContext<'_>, ranged: bool, y: f32, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let kinds = if ranged {
        GuardKind::ranged_all()
    } else {
        GuardKind::melee_all()
    };
    let hired = kinds
        .iter()
        .copied()
        .filter(|kind| ctx.session.campaign.is_guard_hired(*kind))
        .collect::<Vec<_>>();

    for (index, kind) in hired.iter().enumerate() {
        let rect = Rect::new(84.0 + index as f32 * 372.0, y, 332.0, 104.0);
        draw_pool_card(ctx, *kind, rect, ranged, mouse, actions);
    }
}

fn draw_pool_card(
    ctx: &UiContext<'_>,
    kind: GuardKind,
    rect: Rect,
    ranged: bool,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let campaign = &ctx.session.campaign;
    let available = campaign.is_guard_available(kind);
    let stars = campaign.guard_star_level(kind);
    let recovery = campaign.guard_recovery_missions(kind);
    let fill = if available {
        PANEL_ALT
    } else {
        Color::new(0.055, 0.060, 0.055, 0.78)
    };
    draw_panel_with_fill(rect, fill, available);
    draw_ui_text_ex(
        kind.label(),
        rect.x + 18.0,
        rect.y + 27.0,
        TextStyle::new(20.0, INK).params(),
    );
    draw_badge(
        Rect::new(rect.x + 18.0, rect.y + 38.0, 66.0, 22.0),
        &format!("{}*", stars),
        Color::new(0.18, 0.16, 0.08, 1.0),
        UI_GOLD,
    );
    draw_ui_text_ex(
        kind.ability_summary(stars),
        rect.x + 94.0,
        rect.y + 56.0,
        TextStyle::new(14.0, MUTED).params(),
    );
    let status = if recovery > 0 {
        format!("Recovering {} mission", recovery)
    } else {
        "Ready".to_owned()
    };
    draw_badge(
        Rect::new(rect.x + 18.0, rect.bottom() - 31.0, 134.0, 23.0),
        &status,
        Color::new(0.13, 0.17, 0.17, 1.0),
        INK,
    );

    let slot_count = visible_slot_count(ctx, ranged);
    for slot in 0..slot_count.min(4) {
        if virtual_button(
            Rect::new(
                rect.right() - 146.0 + slot as f32 * 34.0,
                rect.bottom() - 34.0,
                28.0,
                26.0,
            ),
            &(slot + 1).to_string(),
            available,
            ButtonTone::Positive,
            mouse,
        ) {
            actions.push(if ranged {
                UiAction::AssignRangedSlot(slot, kind.id().to_owned())
            } else {
                UiAction::AssignGuardSlot(slot, kind.id().to_owned())
            });
        }
    }
}

fn visible_slot_count(ctx: &UiContext<'_>, ranged: bool) -> usize {
    let campaign = &ctx.session.campaign;
    let unlocked_slots = if ranged {
        campaign.ranged_slot_count()
    } else {
        campaign.guard_slot_count()
    };
    let purchased = if ranged {
        GuardKind::ranged_all()
            .iter()
            .filter(|kind| campaign.is_guard_hired(**kind))
            .count()
    } else {
        GuardKind::melee_all()
            .iter()
            .filter(|kind| campaign.is_guard_hired(**kind))
            .count()
    };

    purchased.clamp(1, unlocked_slots)
}
