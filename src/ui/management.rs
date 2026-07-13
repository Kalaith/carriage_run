//! Campaign management screens: shop, guard roster, settings, and pause menu.

use super::gameplay;
use super::upgrade_visuals::{
    draw_panel, draw_panel_with_fill, draw_section_label, GOLD as UI_GOLD, GOLD_SOFT, INK, MUTED,
    PANEL, PANEL_ALT,
};
use super::widgets::{draw_menu_backdrop, draw_top_nav, virtual_button};
use super::{UiAction, UiContext, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::state::{DifficultyPreset, GuardKind};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_shop(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(84.0);
    draw_top_nav(ctx, "Hire Shop", mouse, actions);

    let banner = Rect::new(82.0, 122.0, 1116.0, 88.0);
    draw_panel(banner, false);
    draw_section_label("Shop", banner.x + 24.0, banner.y + 18.0, 360.0);
    draw_ui_text_ex(
        "Recruit road guards",
        banner.x + 24.0,
        banner.y + 46.0,
        TextStyle::new(26.0, INK).params(),
    );
    draw_text_block(
        "Hired guards are permanent roster options. Carriage level unlocks tougher recruits.",
        banner.x + 24.0,
        banner.y + 58.0,
        680.0,
        30.0,
        17.0,
        2.0,
        MUTED,
    );
    draw_badge(
        Rect::new(banner.right() - 192.0, banner.y + 31.0, 156.0, 28.0),
        &format!("Gold {}", ctx.session.campaign.gold),
        Color::new(0.28, 0.21, 0.08, 1.0),
        INK,
    );

    for (index, kind) in GuardKind::all().iter().enumerate() {
        let col = index % 3;
        let row = index / 3;
        let rect = Rect::new(
            84.0 + col as f32 * 372.0,
            238.0 + row as f32 * 174.0,
            332.0,
            154.0,
        );
        draw_shop_guard_card(ctx, *kind, rect, mouse, actions);
    }

    draw_provisions_panel(ctx, Rect::new(622.0, 578.0, 576.0, 58.0), mouse, actions);

    if virtual_button(
        Rect::new(84.0, 592.0, 160.0, 42.0),
        "Guard Roster",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::OpenGuards);
    }
    if virtual_button(
        Rect::new(262.0, 592.0, 160.0, 42.0),
        "Upgrades",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenUpgrades);
    }
    if virtual_button(
        Rect::new(440.0, 592.0, 160.0, 42.0),
        "Carriages",
        true,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::OpenCarriages);
    }
}

pub(super) fn draw_guards(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(104.0);
    draw_top_nav(ctx, "Guard Roster", mouse, actions);

    let active = ctx
        .session
        .campaign
        .selected_melee_kinds()
        .first()
        .copied()
        .unwrap_or_else(|| ctx.session.campaign.selected_guard_kind());
    let detail = Rect::new(84.0, 122.0, 1112.0, 128.0);
    draw_panel(detail, true);
    draw_guard_portrait(vec2(detail.x + 62.0, detail.y + 66.0), active, true);
    draw_ui_text_ex(
        active.label(),
        detail.x + 120.0,
        detail.y + 38.0,
        TextStyle::new(27.0, INK).params(),
    );
    draw_ui_text_ex(
        "Active Guard",
        detail.x + 120.0,
        detail.y + 70.0,
        TextStyle::new(18.0, MUTED).params(),
    );
    draw_ui_text_ex(
        &active.stat_summary(
            ctx.session.campaign.guard_level,
            ctx.session.campaign.archer_level,
            ctx.session.campaign.guard_star_level(active),
        ),
        detail.x + 120.0,
        detail.y + 104.0,
        TextStyle::new(17.0, MUTED).params(),
    );
    draw_badge(
        Rect::new(detail.right() - 184.0, detail.y + 48.0, 144.0, 30.0),
        &format!("Training L{}", ctx.session.campaign.guard_level),
        Color::new(0.13, 0.17, 0.17, 1.0),
        INK,
    );

    for (index, kind) in GuardKind::all().iter().enumerate() {
        let col = index % 3;
        let row = index / 3;
        let rect = Rect::new(
            84.0 + col as f32 * 372.0,
            282.0 + row as f32 * 186.0,
            332.0,
            170.0,
        );
        draw_roster_guard_card(ctx, *kind, rect, mouse, actions);
    }
}

pub(super) fn draw_settings(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    if ctx.session.mission.is_some() {
        let mut ignored = Vec::new();
        gameplay::draw_gameplay(ctx, mouse, &mut ignored);
        draw_dim_overlay();
        draw_settings_panel(
            ctx,
            mouse,
            actions,
            Rect::new(390.0, 74.0, 500.0, 566.0),
            true,
        );
    } else {
        draw_menu_backdrop(126.0);
        draw_top_nav(ctx, "Settings", mouse, actions);
        draw_settings_panel(
            ctx,
            mouse,
            actions,
            Rect::new(360.0, 96.0, 560.0, 566.0),
            false,
        );
    }
}

pub(super) fn draw_pause(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    if ctx.session.mission.is_some() {
        let mut ignored = Vec::new();
        gameplay::draw_gameplay(ctx, mouse, &mut ignored);
    } else {
        draw_menu_backdrop(0.0);
    }

    draw_dim_overlay();
    let panel = Rect::new(420.0, 156.0, 440.0, 382.0);
    draw_panel(panel, true);
    draw_text_centered_in_box(
        "Paused",
        panel.x + 36.0,
        panel.y + 34.0,
        panel.w - 72.0,
        44.0,
        38.0,
        INK,
    );

    if let Some(run) = &ctx.session.mission {
        draw_text_centered_in_box(
            &run.mission_name,
            panel.x + 38.0,
            panel.y + 84.0,
            panel.w - 76.0,
            28.0,
            21.0,
            MUTED,
        );
    }

    let mut y = panel.y + 138.0;
    if virtual_button(
        Rect::new(panel.x + 78.0, y, panel.w - 156.0, 42.0),
        "Resume",
        ctx.session.mission.is_some(),
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::ResumeGame);
    }
    y += 56.0;
    if virtual_button(
        Rect::new(panel.x + 78.0, y, panel.w - 156.0, 42.0),
        "Settings",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::OpenSettings);
    }
    y += 56.0;
    if virtual_button(
        Rect::new(panel.x + 78.0, y, panel.w - 156.0, 42.0),
        "Save",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::Save);
    }
    y += 56.0;
    if virtual_button(
        Rect::new(panel.x + 78.0, y, panel.w - 156.0, 42.0),
        "Route Map",
        true,
        ButtonTone::Muted,
        mouse,
    ) {
        actions.push(UiAction::OpenMap);
    }
}

/// Consumable provisions: a repeatable gold sink. Currently one item, the
/// Reinforced Kit (a one-route health boost spent on mission start).
fn draw_provisions_panel(
    ctx: &UiContext<'_>,
    rect: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    use crate::state::REINFORCED_KIT_COST;
    draw_panel_with_fill(rect, PANEL_ALT, false);
    draw_ui_text_ex(
        "Reinforced Kit",
        rect.x + 18.0,
        rect.y + 26.0,
        TextStyle::new(19.0, INK).params(),
    );
    draw_ui_text_ex(
        "+55 carriage health for one route",
        rect.x + 18.0,
        rect.y + 46.0,
        TextStyle::new(13.0, MUTED).params(),
    );
    draw_badge(
        Rect::new(rect.right() - 268.0, rect.y + 17.0, 106.0, 26.0),
        &format!("Stock {}", ctx.session.campaign.reinforced_kits),
        Color::new(0.13, 0.17, 0.17, 1.0),
        INK,
    );
    let can_buy = ctx.session.campaign.gold >= REINFORCED_KIT_COST;
    if virtual_button(
        Rect::new(rect.right() - 148.0, rect.y + 12.0, 132.0, 34.0),
        &format!("Buy {REINFORCED_KIT_COST}g"),
        can_buy,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::BuyReinforcedKit);
    }
}

fn draw_shop_guard_card(
    ctx: &UiContext<'_>,
    kind: GuardKind,
    rect: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let campaign = &ctx.session.campaign;
    let hired = campaign.is_guard_hired(kind);
    let unlocked = campaign.is_guard_unlocked(kind);
    let selected = campaign.selected_guard_ids.iter().any(|id| id == kind.id())
        || campaign
            .selected_ranged_ids
            .iter()
            .any(|id| id == kind.id());
    let can_hire = campaign.can_hire_guard(kind);
    let stars = campaign.guard_star_level(kind);

    draw_guard_card_shell(rect, selected, hired, unlocked);
    draw_guard_portrait(vec2(rect.x + 46.0, rect.y + 50.0), kind, hired);
    draw_ui_text_ex(
        kind.label(),
        rect.x + 92.0,
        rect.y + 31.0,
        TextStyle::new(20.0, INK).params(),
    );
    draw_text_block(
        kind.description(),
        rect.x + 22.0,
        rect.y + 70.0,
        rect.w - 44.0,
        34.0,
        14.0,
        2.0,
        MUTED,
    );
    draw_ui_text_ex(
        kind.ability_summary(stars),
        rect.x + 22.0,
        rect.y + 116.0,
        TextStyle::new(14.0, MUTED).params(),
    );

    let status = if selected {
        "Active".to_owned()
    } else if hired {
        "Hired".to_owned()
    } else if unlocked {
        format!("{} gold", campaign.guard_hire_cost(kind))
    } else {
        format!("Carriage L{}", kind.unlock_level())
    };
    draw_badge(
        Rect::new(rect.x + 22.0, rect.bottom() - 33.0, 132.0, 24.0),
        &status,
        Color::new(0.13, 0.17, 0.17, 1.0),
        INK,
    );

    let (label, enabled, action) = if hired {
        (
            if selected { "Active" } else { "Set Active" },
            !selected,
            UiAction::SelectGuard(kind.id().to_owned()),
        )
    } else if unlocked {
        (
            if campaign.gold >= campaign.guard_hire_cost(kind) {
                "Hire"
            } else {
                "Need Gold"
            },
            can_hire,
            UiAction::HireGuard(kind.id().to_owned()),
        )
    } else {
        ("Locked", false, UiAction::OpenUpgrades)
    };

    if virtual_button(
        Rect::new(rect.right() - 122.0, rect.bottom() - 38.0, 92.0, 30.0),
        label,
        enabled,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(action);
    }
}

fn draw_roster_guard_card(
    ctx: &UiContext<'_>,
    kind: GuardKind,
    rect: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let campaign = &ctx.session.campaign;
    let hired = campaign.is_guard_hired(kind);
    let unlocked = campaign.is_guard_unlocked(kind);
    let selected = campaign.selected_guard_ids.iter().any(|id| id == kind.id())
        || campaign
            .selected_ranged_ids
            .iter()
            .any(|id| id == kind.id());
    let stars = campaign.guard_star_level(kind);
    let recovery = campaign.guard_recovery_missions(kind);

    draw_guard_card_shell(rect, selected, hired, unlocked);
    draw_guard_portrait(vec2(rect.x + 44.0, rect.y + 48.0), kind, hired);
    draw_ui_text_ex(
        kind.label(),
        rect.x + 90.0,
        rect.y + 29.0,
        TextStyle::new(20.0, INK).params(),
    );
    draw_text_block(
        kind.ability_summary(stars),
        rect.x + 22.0,
        rect.y + 72.0,
        rect.w - 44.0,
        34.0,
        14.0,
        2.0,
        MUTED,
    );
    draw_ui_text_ex(
        &kind.stat_summary(campaign.guard_level, campaign.archer_level, stars),
        rect.x + 22.0,
        rect.y + 122.0,
        TextStyle::new(13.0, MUTED).params(),
    );

    // When a guard is recovering, the primary action becomes paying the
    // infirmary to bring them back early.
    let (label, enabled, action, tone) =
        if let Some(cost) = campaign.guard_treat_cost(kind).filter(|_| hired) {
            (
                format!("Treat {}", cost),
                campaign.gold >= cost,
                UiAction::TreatGuard(kind.id().to_owned()),
                ButtonTone::Positive,
            )
        } else if selected {
            (
                "Active".to_owned(),
                false,
                UiAction::SelectGuard(kind.id().to_owned()),
                ButtonTone::Positive,
            )
        } else if hired {
            (
                "Set Active".to_owned(),
                true,
                UiAction::SelectGuard(kind.id().to_owned()),
                ButtonTone::Positive,
            )
        } else {
            (
                "Shop".to_owned(),
                true,
                UiAction::OpenShop,
                ButtonTone::Secondary,
            )
        };
    if virtual_button(
        Rect::new(rect.right() - 224.0, rect.bottom() - 38.0, 88.0, 30.0),
        &label,
        enabled,
        tone,
        mouse,
    ) {
        actions.push(action);
    }

    let status = if recovery > 0 {
        format!("Rest {}", recovery)
    } else if hired {
        "Hired".to_owned()
    } else if unlocked {
        format!("{} gold", campaign.guard_hire_cost(kind))
    } else {
        format!("Carriage L{}", kind.unlock_level())
    };
    draw_badge(
        Rect::new(rect.x + 22.0, rect.bottom() - 33.0, 92.0, 24.0),
        &status,
        Color::new(0.13, 0.17, 0.17, 1.0),
        INK,
    );

    let star_label = campaign
        .guard_star_upgrade_cost(kind)
        .map(|cost| format!("Star {}", cost))
        .unwrap_or_else(|| "3 Star".to_owned());
    if virtual_button(
        Rect::new(rect.right() - 126.0, rect.bottom() - 38.0, 96.0, 30.0),
        &star_label,
        hired
            && campaign.guard_star_upgrade_cost(kind).is_some()
            && campaign.gold >= campaign.guard_star_upgrade_cost(kind).unwrap_or(0),
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::UpgradeGuardStar(kind.id().to_owned()));
    }
}

fn draw_guard_card_shell(rect: Rect, selected: bool, hired: bool, unlocked: bool) {
    let accent = if selected {
        UI_GOLD
    } else if hired {
        GOLD_SOFT
    } else if unlocked {
        Color::new(0.46, 0.42, 0.28, 0.68)
    } else {
        Color::new(0.24, 0.24, 0.20, 0.58)
    };
    let fill = if hired { PANEL_ALT } else { PANEL };
    draw_panel_with_fill(rect, fill, selected);
    draw_rectangle(rect.x, rect.y + 16.0, 4.0, rect.h - 32.0, accent);
}

pub(super) fn draw_guard_portrait(pos: Vec2, kind: GuardKind, enabled: bool) {
    let body = match kind {
        GuardKind::Swordsman => Color::new(0.18, 0.42, 0.64, 1.0),
        GuardKind::ShieldGuard => Color::new(0.20, 0.46, 0.36, 1.0),
        GuardKind::Spearman => Color::new(0.42, 0.34, 0.64, 1.0),
        GuardKind::Archer => Color::new(0.18, 0.46, 0.28, 1.0),
        GuardKind::CrossbowGuard => Color::new(0.42, 0.38, 0.32, 1.0),
        GuardKind::Mage => Color::new(0.26, 0.34, 0.68, 1.0),
    };
    let alpha = if enabled { 1.0 } else { 0.42 };
    draw_circle(
        pos.x + 4.0,
        pos.y + 6.0,
        29.0,
        Color::new(0.0, 0.0, 0.0, 0.22),
    );
    draw_circle(
        pos.x,
        pos.y,
        26.0,
        Color::new(body.r, body.g, body.b, alpha),
    );
    draw_rectangle(
        pos.x - 7.0,
        pos.y - 34.0,
        14.0,
        20.0,
        Color::new(0.76, 0.66, 0.45, alpha),
    );
    match kind {
        GuardKind::Swordsman => draw_line(
            pos.x + 18.0,
            pos.y - 4.0,
            pos.x + 40.0,
            pos.y - 28.0,
            4.0,
            Color::new(0.86, 0.88, 0.82, alpha),
        ),
        GuardKind::ShieldGuard => {
            draw_circle(
                pos.x + 26.0,
                pos.y + 2.0,
                15.0,
                Color::new(0.64, 0.70, 0.64, alpha),
            );
            draw_circle_lines(
                pos.x + 26.0,
                pos.y + 2.0,
                15.0,
                2.0,
                Color::new(0.22, 0.26, 0.22, alpha),
            );
        }
        GuardKind::Spearman => draw_line(
            pos.x - 22.0,
            pos.y + 15.0,
            pos.x + 44.0,
            pos.y - 34.0,
            4.0,
            Color::new(0.82, 0.72, 0.45, alpha),
        ),
        GuardKind::Archer => draw_line(
            pos.x - 16.0,
            pos.y - 2.0,
            pos.x + 23.0,
            pos.y - 18.0,
            3.0,
            Color::new(0.95, 0.80, 0.38, alpha),
        ),
        GuardKind::CrossbowGuard => {
            draw_rectangle(
                pos.x + 14.0,
                pos.y - 12.0,
                28.0,
                8.0,
                Color::new(0.75, 0.70, 0.58, alpha),
            );
            draw_line(
                pos.x + 18.0,
                pos.y - 22.0,
                pos.x + 37.0,
                pos.y + 5.0,
                2.0,
                Color::new(0.86, 0.82, 0.70, alpha),
            );
        }
        GuardKind::Mage => {
            draw_circle(
                pos.x + 24.0,
                pos.y - 20.0,
                8.0,
                Color::new(0.58, 0.86, 1.0, alpha),
            );
            draw_line(
                pos.x + 16.0,
                pos.y - 4.0,
                pos.x + 28.0,
                pos.y - 28.0,
                3.0,
                Color::new(0.68, 0.52, 0.30, alpha),
            );
        }
    }
}

fn draw_settings_panel(
    ctx: &UiContext<'_>,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    in_mission: bool,
) {
    draw_panel(panel, true);
    draw_ui_text_ex(
        "Settings",
        panel.x + 34.0,
        panel.y + 46.0,
        TextStyle::new(33.0, INK).params(),
    );

    let mut y = panel.y + 94.0;
    draw_setting_row(
        "route_motion",
        "Route Motion",
        ctx.session.campaign.route_motion_enabled,
        Rect::new(panel.x + 34.0, y, panel.w - 68.0, 58.0),
        mouse,
        actions,
    );
    y += 72.0;
    draw_setting_row(
        "alerts",
        "Route Alerts",
        ctx.session.campaign.alerts_enabled,
        Rect::new(panel.x + 34.0, y, panel.w - 68.0, 58.0),
        mouse,
        actions,
    );
    y += 72.0;
    draw_setting_row(
        "auto_save",
        "Autosave",
        ctx.session.campaign.auto_save_enabled,
        Rect::new(panel.x + 34.0, y, panel.w - 68.0, 58.0),
        mouse,
        actions,
    );
    y += 72.0;
    draw_difficulty_row(
        ctx,
        Rect::new(panel.x + 34.0, y, panel.w - 68.0, 58.0),
        mouse,
        actions,
    );
    y += 72.0;
    draw_setting_row(
        "generous_timers",
        "Generous Timers",
        ctx.session.campaign.generous_timers,
        Rect::new(panel.x + 34.0, y, panel.w - 68.0, 58.0),
        mouse,
        actions,
    );

    let button_y = panel.bottom() - 72.0;
    if in_mission {
        if virtual_button(
            Rect::new(panel.x + 68.0, button_y, panel.w - 136.0, 42.0),
            "Resume",
            true,
            ButtonTone::Positive,
            mouse,
        ) {
            actions.push(UiAction::ResumeGame);
        }
    } else {
        if virtual_button(
            Rect::new(panel.x + 74.0, button_y, 130.0, 42.0),
            "Map",
            true,
            ButtonTone::Primary,
            mouse,
        ) {
            actions.push(UiAction::OpenMap);
        }
        if virtual_button(
            Rect::new(panel.right() - 204.0, button_y, 130.0, 42.0),
            "Title",
            true,
            ButtonTone::Muted,
            mouse,
        ) {
            actions.push(UiAction::ReturnTitle);
        }
    }
}

fn draw_difficulty_row(ctx: &UiContext<'_>, rect: Rect, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_panel_with_fill(rect, PANEL_ALT, true);
    draw_ui_text_ex(
        "Difficulty",
        rect.x + 18.0,
        rect.y + 36.0,
        TextStyle::new(20.0, INK).params(),
    );

    let current = ctx.session.campaign.difficulty_preset;
    let presets = DifficultyPreset::all();
    let button_w = 96.0;
    let gap = 8.0;
    let total = presets.len() as f32 * button_w + (presets.len() as f32 - 1.0) * gap;
    let mut x = rect.right() - 20.0 - total;
    for preset in presets {
        let active = preset == current;
        if virtual_button(
            Rect::new(x, rect.y + 12.0, button_w, 34.0),
            preset.label(),
            true,
            if active {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            mouse,
        ) {
            actions.push(UiAction::SetDifficulty(preset.id().to_owned()));
        }
        x += button_w + gap;
    }
}

fn draw_setting_row(
    id: &str,
    label: &str,
    enabled: bool,
    rect: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_panel_with_fill(rect, PANEL_ALT, enabled);
    draw_ui_text_ex(
        label,
        rect.x + 18.0,
        rect.y + 36.0,
        TextStyle::new(20.0, INK).params(),
    );
    draw_badge(
        Rect::new(rect.right() - 182.0, rect.y + 16.0, 74.0, 26.0),
        if enabled { "On" } else { "Off" },
        if enabled {
            Color::new(0.12, 0.24, 0.15, 1.0)
        } else {
            Color::new(0.20, 0.14, 0.12, 1.0)
        },
        INK,
    );
    if virtual_button(
        Rect::new(rect.right() - 98.0, rect.y + 12.0, 78.0, 34.0),
        "Toggle",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::ToggleSetting(id.to_owned()));
    }
}

fn draw_dim_overlay() {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.54),
    );
}
