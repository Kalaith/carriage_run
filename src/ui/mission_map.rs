//! Routes screen and mission selection layout.

use super::mission_map_art::{
    draw_cargo_icon, draw_meter_bar, draw_mini_icon, draw_mission_status, draw_mission_thumbnail,
    draw_reward, draw_route_icon, draw_star, draw_type_badge,
};
use super::upgrade_visuals::{
    draw_crest, draw_panel_with_fill, draw_stat_icon, nav_tile, GOLD as UI_GOLD, GOLD_SOFT, INK,
    MUTED,
};
use super::widgets::{draw_menu_backdrop, format_label};
use super::{UiAction, UiContext};
use crate::data::{MissionDef, RouteChoiceDef};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_mission_map(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(36.0);
    draw_routes_header(ctx, mouse, actions);

    let missions = ctx.data.missions_ordered();
    let campaign = &ctx.session.campaign;
    let visible: Vec<&MissionDef> = missions
        .iter()
        .copied()
        .filter(|mission| {
            campaign.is_mission_unlocked(mission) || campaign.is_mission_near_unlock(mission)
        })
        .collect();
    for (index, mission) in visible.iter().enumerate() {
        let col = index % 4;
        let row = index / 4;
        let rect = Rect::new(
            50.0 + col as f32 * 296.0,
            124.0 + row as f32 * 106.0,
            280.0,
            92.0,
        );
        if campaign.is_mission_unlocked(mission) {
            if draw_mission_card(ctx, mission, rect, mouse) {
                actions.push(UiAction::SelectMission(mission.id.clone()));
            }
        } else {
            draw_teaser_card(ctx, mission, rect);
        }
    }

    let selected = ctx
        .data
        .missions
        .get(&ctx.session.campaign.selected_mission_id)
        .or_else(|| missions.first().copied());
    if let Some(mission) = selected {
        draw_selected_route(ctx, mission, mouse, actions);
    }
}

fn draw_routes_header(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_crest(Rect::new(34.0, 18.0, 84.0, 78.0));
    draw_ui_text_ex("Routes", 138.0, 72.0, TextStyle::new(54.0, INK).params());

    let gold = Rect::new(306.0, 24.0, 116.0, 62.0);
    draw_panel_with_fill(gold, Color::new(0.040, 0.050, 0.043, 0.95), false);
    draw_stat_icon("gold", vec2(gold.x + 28.0, gold.y + 34.0));
    draw_ui_text_ex(
        "Gold",
        gold.x + 56.0,
        gold.y + 22.0,
        TextStyle::new(13.0, UI_GOLD).params(),
    );
    draw_ui_text_ex(
        &ctx.session.campaign.gold.to_string(),
        gold.x + 56.0,
        gold.y + 48.0,
        TextStyle::new(23.0, INK).params(),
    );

    let level = Rect::new(432.0, 24.0, 210.0, 62.0);
    draw_panel_with_fill(level, Color::new(0.040, 0.050, 0.043, 0.95), false);
    draw_stat_icon("level", vec2(level.x + 32.0, level.y + 33.0));
    draw_ui_text_ex(
        "Carriage Level",
        level.x + 70.0,
        level.y + 22.0,
        TextStyle::new(13.0, UI_GOLD).params(),
    );
    draw_ui_text_ex(
        &ctx.session.campaign.carriage_level.to_string(),
        level.x + 30.0,
        level.y + 44.0,
        TextStyle::new(20.0, INK).params(),
    );
    let xp = route_xp(ctx);
    draw_meter_bar(
        Rect::new(level.x + 74.0, level.y + 40.0, 92.0, 10.0),
        xp as f32 / 300.0,
        UI_GOLD,
    );
    draw_ui_text_ex(
        &format!("{} / 300 XP", xp),
        level.x + 170.0,
        level.y + 49.0,
        TextStyle::new(10.0, MUTED).params(),
    );

    let nav = [
        ("Map", "map", UiAction::OpenMap, true),
        ("Shop", "shop", UiAction::OpenShop, false),
        ("Guards", "guards", UiAction::OpenGuards, false),
        ("Upgrades", "up", UiAction::OpenUpgrades, false),
        ("Settings", "settings", UiAction::OpenSettings, false),
    ];
    let mut x = 678.0;
    for (label, icon, action, active) in nav {
        if nav_tile(Rect::new(x, 20.0, 94.0, 72.0), label, icon, active, mouse) {
            actions.push(action);
        }
        x += 104.0;
    }
}

fn route_xp(ctx: &UiContext<'_>) -> i64 {
    let completed = ctx
        .session
        .campaign
        .records
        .values()
        .map(|record| record.completions as i64)
        .sum::<i64>();
    (completed * 140).clamp(0, 300)
}

fn draw_mission_card(ctx: &UiContext<'_>, mission: &MissionDef, rect: Rect, mouse: Vec2) -> bool {
    let campaign = &ctx.session.campaign;
    let selected = campaign.selected_mission_id == mission.id;
    let unlocked = campaign.is_mission_unlocked(mission);
    let hovered = unlocked && rect.contains_point(mouse);
    let fill = if selected {
        Color::new(0.070, 0.120, 0.085, 0.98)
    } else if hovered {
        Color::new(0.090, 0.105, 0.078, 0.98)
    } else {
        Color::new(0.046, 0.056, 0.050, 0.96)
    };
    draw_panel_with_fill(rect, fill, selected);

    if selected {
        draw_selected_tab(rect);
    }

    draw_mission_thumbnail(Rect::new(rect.x + 13.0, rect.y + 12.0, 66.0, 68.0), mission);
    draw_text_block(
        &mission.name,
        rect.x + 94.0,
        rect.y + 17.0,
        rect.w - 176.0,
        32.0,
        18.0,
        2.0,
        INK,
    );
    draw_ui_text_ex(
        &mission.route,
        rect.x + 94.0,
        rect.y + 55.0,
        TextStyle::new(15.0, Color::new(0.58, 0.82, 0.36, 1.0)).params(),
    );
    draw_type_badge(
        Rect::new(rect.right() - 80.0, rect.y + 17.0, 62.0, 26.0),
        &mission.mission_type,
    );
    draw_line(
        rect.x + 94.0,
        rect.y + 66.0,
        rect.right() - 16.0,
        rect.y + 66.0,
        1.0,
        Color::new(0.55, 0.40, 0.18, 0.36),
    );
    draw_reward(
        vec2(rect.x + 98.0, rect.bottom() - 17.0),
        effective_reward(ctx, mission),
        14.0,
    );
    draw_mission_status(
        Rect::new(rect.right() - 100.0, rect.bottom() - 30.0, 84.0, 22.0),
        unlocked,
        &campaign.mission_unlock_label(mission),
    );

    if !unlocked {
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.0, 0.0, 0.0, 0.44),
        );
        draw_mission_status(
            Rect::new(rect.right() - 108.0, rect.bottom() - 30.0, 92.0, 22.0),
            false,
            &campaign.mission_unlock_label(mission),
        );
    }

    hovered && is_mouse_button_released(MouseButton::Left)
}

fn draw_teaser_card(ctx: &UiContext<'_>, mission: &MissionDef, rect: Rect) {
    draw_panel_with_fill(rect, Color::new(0.040, 0.045, 0.043, 0.96), false);

    let thumb = Rect::new(rect.x + 13.0, rect.y + 12.0, 66.0, 68.0);
    draw_panel_with_fill(thumb, Color::new(0.060, 0.070, 0.060, 0.96), false);
    draw_text_centered_in_box("?", thumb.x, thumb.y, thumb.w, thumb.h, 40.0, MUTED);

    draw_ui_text_ex(
        "Undiscovered Route",
        rect.x + 94.0,
        rect.y + 34.0,
        TextStyle::new(17.0, MUTED).params(),
    );
    draw_ui_text_ex(
        "Locked",
        rect.x + 94.0,
        rect.y + 55.0,
        TextStyle::new(14.0, Color::new(0.58, 0.50, 0.30, 1.0)).params(),
    );
    draw_line(
        rect.x + 94.0,
        rect.y + 66.0,
        rect.right() - 16.0,
        rect.y + 66.0,
        1.0,
        Color::new(0.55, 0.40, 0.18, 0.36),
    );
    draw_mission_status(
        Rect::new(rect.right() - 108.0, rect.bottom() - 30.0, 92.0, 22.0),
        false,
        &ctx.session.campaign.mission_unlock_label(mission),
    );
}

fn draw_selected_tab(rect: Rect) {
    let x = rect.x - 23.0;
    let y = rect.y + 28.0;
    draw_rectangle(x, y - 17.0, 22.0, 34.0, Color::new(0.10, 0.12, 0.07, 0.95));
    draw_rectangle_lines(x, y - 17.0, 22.0, 34.0, 1.0, UI_GOLD);
    draw_star(vec2(x + 11.0, y), 9.0, UI_GOLD);
}

fn draw_selected_route(
    ctx: &UiContext<'_>,
    mission: &MissionDef,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let layout = selected_route_layout(screen_width(), screen_height());
    let panel = layout.panel;
    draw_panel_with_fill(panel, Color::new(0.035, 0.050, 0.044, 0.98), true);

    draw_mission_thumbnail(
        Rect::new(panel.x + 30.0, panel.y + 30.0, 138.0, 170.0),
        mission,
    );
    draw_route_detail_text(
        ctx,
        mission,
        Rect::new(panel.x + 188.0, panel.y + 28.0, 246.0, 178.0),
    );
    draw_divider(panel.x + 456.0, panel);

    draw_route_choices(ctx, mission, layout.route_choices, mouse, actions);
    draw_divider(layout.route_divider, panel);

    let (enemy_mix, hazard_mix) = effective_mixes(ctx, mission);
    draw_mix_tiles(layout.threats, "Threats", &enemy_mix, false);
    draw_mix_tiles(layout.hazards, "Hazards", &hazard_mix, true);
    draw_divider(layout.cta_divider, panel);

    if prepare_loadout_button(
        layout.cta,
        ctx.session.campaign.is_mission_unlocked(mission),
        mouse,
    ) {
        actions.push(UiAction::OpenLoadout);
    }
}

#[derive(Clone, Copy)]
struct SelectedRouteLayout {
    panel: Rect,
    route_choices: Rect,
    route_divider: f32,
    threats: Rect,
    hazards: Rect,
    cta_divider: f32,
    cta: Rect,
}

fn selected_route_layout(screen_w: f32, screen_h: f32) -> SelectedRouteLayout {
    let panel_width = (screen_w - 88.0).clamp(980.0, 1192.0);
    let panel_x = ((screen_w - panel_width) * 0.5).max(20.0);
    let panel_y = (screen_h - 248.0).clamp(424.0, 452.0);
    let panel = Rect::new(panel_x, panel_y, panel_width, 236.0);

    let cta_width = if panel.w < 1160.0 { 128.0 } else { 134.0 };
    let cta = Rect::new(
        panel.right() - cta_width - 24.0,
        panel.y + 82.0,
        cta_width,
        80.0,
    );
    let cta_divider = cta.x - 18.0;
    let mix_width = if panel.w < 1160.0 { 94.0 } else { 104.0 };
    let hazards = Rect::new(
        cta_divider - mix_width - 14.0,
        panel.y + 40.0,
        mix_width,
        150.0,
    );
    let threats = Rect::new(
        hazards.x - mix_width - 14.0,
        panel.y + 40.0,
        mix_width,
        150.0,
    );
    let route_divider = threats.x - 18.0;
    let route_x = panel.x + 466.0;
    let route_width = (route_divider - route_x - 18.0).max(204.0);
    let route_choices = Rect::new(route_x, panel.y + 42.0, route_width, 142.0);

    SelectedRouteLayout {
        panel,
        route_choices,
        route_divider,
        threats,
        hazards,
        cta_divider,
        cta,
    }
}

fn draw_divider(x: f32, panel: Rect) {
    draw_line(x, panel.y + 28.0, x, panel.bottom() - 28.0, 1.0, GOLD_SOFT);
}

fn draw_route_detail_text(ctx: &UiContext<'_>, mission: &MissionDef, rect: Rect) {
    draw_ui_text_ex(
        &mission.name,
        rect.x,
        rect.y + 27.0,
        TextStyle::new(28.0, INK).params(),
    );
    draw_type_badge(
        Rect::new(rect.x, rect.y + 43.0, 74.0, 27.0),
        &mission.mission_type,
    );
    draw_ui_text_ex(
        &mission.route,
        rect.x + 88.0,
        rect.y + 63.0,
        TextStyle::new(17.0, Color::new(0.58, 0.82, 0.36, 1.0)).params(),
    );
    draw_ui_text_ex(
        &mission.cargo,
        rect.x,
        rect.y + 92.0,
        TextStyle::new(26.0, UI_GOLD).params(),
    );
    draw_text_block(
        &mission.objective,
        rect.x,
        rect.y + 102.0,
        rect.w,
        38.0,
        17.0,
        3.0,
        MUTED,
    );
    draw_line(
        rect.x,
        rect.y + 144.0,
        rect.x + rect.w,
        rect.y + 144.0,
        1.0,
        GOLD_SOFT,
    );
    draw_ui_text_ex(
        "Bonus:",
        rect.x,
        rect.y + 162.0,
        TextStyle::new(15.0, UI_GOLD).params(),
    );
    draw_text_block(
        &mission.bonus_objective,
        rect.x + 58.0,
        rect.y + 148.0,
        rect.w - 58.0,
        22.0,
        14.0,
        2.0,
        MUTED,
    );

    let footer = Rect::new(rect.x, rect.y + 176.0, rect.w, 28.0);
    draw_panel_with_fill(footer, Color::new(0.050, 0.060, 0.046, 0.92), false);
    draw_reward(
        vec2(footer.x + 26.0, footer.y + 19.0),
        effective_reward(ctx, mission),
        13.0,
    );
    draw_mission_status(
        Rect::new(footer.right() - 100.0, footer.y + 5.0, 90.0, 18.0),
        ctx.session.campaign.is_mission_unlocked(mission),
        &ctx.session.campaign.mission_unlock_label(mission),
    );
}

fn draw_route_choices(
    ctx: &UiContext<'_>,
    mission: &MissionDef,
    rect: Rect,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_compact_section_label(
        "Route Choice",
        Rect::new(rect.x + 4.0, rect.y - 10.0, rect.w - 8.0, 24.0),
    );
    let selected_id = ctx.session.campaign.selected_route_choice_id(mission);
    let visible_choices = mission.route_choices.iter().take(2).count().max(1);
    let gap = 8.0;
    let button_width =
        (rect.w - gap * (visible_choices.saturating_sub(1) as f32)) / visible_choices as f32;
    for (index, choice) in mission.route_choices.iter().take(2).enumerate() {
        let button = Rect::new(
            rect.x + index as f32 * (button_width + gap),
            rect.y + 24.0,
            button_width,
            42.0,
        );
        let selected = selected_id == Some(choice.id.as_str());
        if draw_route_choice_button(button, choice, selected, mouse) {
            actions.push(UiAction::SelectRouteChoice(choice.id.clone()));
        }
    }

    let selected = ctx.session.campaign.selected_route_choice(mission);
    let description = selected
        .map(|choice| choice.description.as_str())
        .unwrap_or(mission.route.as_str());
    draw_text_block(
        description,
        rect.x + 8.0,
        rect.y + 84.0,
        rect.w - 16.0,
        34.0,
        15.0,
        2.0,
        MUTED,
    );
    draw_route_stats(
        ctx,
        mission,
        Rect::new(rect.x + 4.0, rect.y + 132.0, rect.w - 8.0, 42.0),
    );
}

fn draw_compact_section_label(label: &str, rect: Rect) {
    draw_line(
        rect.x,
        rect.y + rect.h * 0.5,
        rect.x + 82.0,
        rect.y + rect.h * 0.5,
        1.0,
        GOLD_SOFT,
    );
    draw_line(
        rect.right() - 82.0,
        rect.y + rect.h * 0.5,
        rect.right(),
        rect.y + rect.h * 0.5,
        1.0,
        GOLD_SOFT,
    );
    draw_text_centered_in_box(
        label,
        rect.x + 88.0,
        rect.y + 2.0,
        rect.w - 176.0,
        rect.h - 4.0,
        14.0,
        UI_GOLD,
    );
}

fn draw_route_choice_button(
    rect: Rect,
    choice: &RouteChoiceDef,
    selected: bool,
    mouse: Vec2,
) -> bool {
    let hovered = rect.contains_point(mouse);
    draw_panel_with_fill(
        rect,
        if selected {
            Color::new(0.060, 0.160, 0.090, 0.96)
        } else if hovered {
            Color::new(0.105, 0.095, 0.066, 0.96)
        } else {
            Color::new(0.050, 0.052, 0.046, 0.92)
        },
        selected,
    );
    draw_route_icon(vec2(rect.x + 22.0, rect.y + 22.0), selected);
    draw_text_centered_in_box(
        &choice.name,
        rect.x + 42.0,
        rect.y + 7.0,
        rect.w - 48.0,
        rect.h - 14.0,
        15.0,
        INK,
    );
    hovered && is_mouse_button_released(MouseButton::Left)
}

fn draw_route_stats(ctx: &UiContext<'_>, mission: &MissionDef, rect: Rect) {
    let half = (rect.w - 8.0) * 0.5;
    let distance = Rect::new(rect.x, rect.y, half, rect.h);
    let weight = Rect::new(rect.x + half + 8.0, rect.y, half, rect.h);
    draw_panel_with_fill(distance, Color::new(0.045, 0.050, 0.042, 0.94), false);
    draw_panel_with_fill(weight, Color::new(0.045, 0.050, 0.042, 0.94), false);
    draw_route_icon(vec2(distance.x + 20.0, distance.y + 22.0), true);
    draw_stat_text(
        distance,
        "Distance",
        &format!("{:.0} m", effective_distance(ctx, mission)),
    );
    draw_cargo_icon(vec2(weight.x + 20.0, weight.y + 22.0));
    draw_stat_text(
        weight,
        "Cargo Weight",
        &format!("{:.1}x", mission.difficulty),
    );
}

fn draw_stat_text(rect: Rect, label: &str, value: &str) {
    draw_text_centered_in_box(
        label,
        rect.x + 44.0,
        rect.y + 8.0,
        rect.w - 50.0,
        14.0,
        11.0,
        UI_GOLD,
    );
    draw_text_centered_in_box(
        value,
        rect.x + 44.0,
        rect.y + 24.0,
        rect.w - 50.0,
        18.0,
        15.0,
        INK,
    );
}

fn draw_mix_tiles(rect: Rect, title: &str, values: &[String], hazard: bool) {
    draw_text_centered_in_box(title, rect.x, rect.y, rect.w, 20.0, 16.0, UI_GOLD);
    for (index, value) in compact_mix(values.to_vec(), 4).iter().enumerate() {
        let tile = Rect::new(rect.x, rect.y + 32.0 + index as f32 * 35.0, rect.w, 29.0);
        let color = if hazard {
            Color::new(0.23, 0.14, 0.045, 0.94)
        } else {
            Color::new(0.23, 0.085, 0.060, 0.94)
        };
        draw_panel_with_fill(tile, color, false);
        draw_mini_icon(vec2(tile.x + 22.0, tile.y + 16.0), value, hazard);
        draw_text_centered_in_box(
            &format_label(value),
            tile.x + 42.0,
            tile.y + 6.0,
            tile.w - 48.0,
            tile.h - 10.0,
            12.0,
            INK,
        );
    }
}

fn prepare_loadout_button(rect: Rect, enabled: bool, mouse: Vec2) -> bool {
    let hovered = enabled && rect.contains_point(mouse);
    draw_panel_with_fill(
        rect,
        if !enabled {
            Color::new(0.06, 0.06, 0.055, 0.82)
        } else if hovered {
            Color::new(0.08, 0.25, 0.13, 0.98)
        } else {
            Color::new(0.060, 0.160, 0.090, 0.98)
        },
        enabled,
    );
    draw_route_icon(vec2(rect.x + rect.w * 0.5, rect.y + 12.0), enabled);
    draw_text_centered_in_box(
        "Prepare Loadout",
        rect.x + 10.0,
        rect.y + 28.0,
        rect.w - 20.0,
        24.0,
        14.0,
        if enabled { INK } else { MUTED },
    );
    draw_text_centered_in_box(
        "Start Route",
        rect.x + 18.0,
        rect.y + 54.0,
        rect.w - 36.0,
        18.0,
        12.0,
        if enabled { UI_GOLD } else { MUTED },
    );
    hovered && is_mouse_button_released(MouseButton::Left)
}

fn selected_route_choice<'a>(
    ctx: &UiContext<'_>,
    mission: &'a MissionDef,
) -> Option<&'a RouteChoiceDef> {
    ctx.session.campaign.selected_route_choice(mission)
}

fn effective_distance(ctx: &UiContext<'_>, mission: &MissionDef) -> f32 {
    selected_route_choice(ctx, mission)
        .map(|choice| mission.distance + choice.distance_delta)
        .unwrap_or(mission.distance)
        .max(420.0)
}

fn effective_reward(ctx: &UiContext<'_>, mission: &MissionDef) -> i64 {
    selected_route_choice(ctx, mission)
        .map(|choice| mission.base_reward + choice.reward_delta)
        .unwrap_or(mission.base_reward)
        .max(0)
}

fn effective_mixes(ctx: &UiContext<'_>, mission: &MissionDef) -> (Vec<String>, Vec<String>) {
    let mut enemies = mission.enemy_mix.clone();
    let mut hazards = mission.hazard_mix.clone();
    if let Some(choice) = selected_route_choice(ctx, mission) {
        enemies.extend(choice.enemy_add.iter().cloned());
        hazards.extend(choice.hazard_add.iter().cloned());
    }
    (compact_mix(enemies, 4), compact_mix(hazards, 4))
}

fn compact_mix(values: Vec<String>, limit: usize) -> Vec<String> {
    if values.len() <= limit {
        return values;
    }

    let hidden = values.len() - limit;
    let mut compact = values
        .into_iter()
        .take(limit.saturating_sub(1))
        .collect::<Vec<_>>();
    compact.push(format!("{} more", hidden + 1));
    compact
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_loadout_button_stays_inside_published_embed_panel() {
        assert_prepare_loadout_is_in_panel(selected_route_layout(1200.0, 675.0), 1200.0);
    }

    #[test]
    fn prepare_loadout_button_stays_inside_standard_panel() {
        assert_prepare_loadout_is_in_panel(selected_route_layout(1280.0, 720.0), 1280.0);
    }

    fn assert_prepare_loadout_is_in_panel(layout: SelectedRouteLayout, screen_w: f32) {
        assert!(layout.panel.x >= 0.0);
        assert!(layout.panel.right() <= screen_w);
        assert!(layout.cta.x > layout.cta_divider);
        assert!(layout.cta.x >= layout.panel.x);
        assert!(layout.cta.right() <= layout.panel.right());
        assert!(layout.cta.bottom() <= layout.panel.bottom());
        assert!(layout.hazards.right() < layout.cta_divider);
        assert!(layout.threats.right() < layout.hazards.x);
        assert!(layout.route_choices.right() < layout.route_divider);
    }
}
