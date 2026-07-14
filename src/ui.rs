//! Immediate-mode UI entry points and menu screens for Carriage Run.

mod carriage;
mod carriages;
mod gameplay;
mod gameplay_hud;
mod journey;
mod loadout;
mod management;
mod mission_map;
mod mission_map_art;
mod outfitter;
mod records;
mod upgrade_visuals;
mod upgrades;
mod widgets;

use crate::data::GameData;
use crate::state::{Screen, PLAY_BOTTOM, PLAY_TOP};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_centered, draw_ui_text_ex};
use upgrade_visuals::{
    draw_crest, draw_panel, draw_section_label, GOLD as UI_GOLD, GOLD_SOFT, INK, MUTED,
};
use widgets::*;

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    NewCampaign,
    RequestNewCampaign,
    DismissConfirm,
    ContinueCampaign,
    OpenMap,
    OpenLoadout,
    OpenShop,
    OpenCarriages,
    OpenGuards,
    OpenUpgrades,
    OpenSettings,
    OpenCodex,
    SetCodexTab(crate::state::CodexTab),
    ReturnTitle,
    PauseGame,
    ResumeGame,
    SelectMission(String),
    SelectRouteChoice(String),
    SelectGuard(String),
    AssignGuardSlot(usize, String),
    ClearGuardSlot(usize),
    AssignRangedSlot(usize, String),
    ClearRangedSlot(usize),
    AssignEquipmentSlot(usize, String),
    ClearEquipmentSlot(usize),
    HireGuard(String),
    UpgradeGuardStar(String),
    TreatGuard(String),
    ToggleSetting(String),
    SetDifficulty(String),
    BeginMission,
    OpenOutfitter,
    OpenRecords,
    SelectStake(String),
    UnlockStartingRelic(String),
    StartExpedition,
    StartDailyExpedition,
    JourneyPressOn,
    JourneyChooseReward(usize),
    JourneyResolveEvent(usize),
    JourneyBeginLeg(usize),
    JourneyRepair,
    JourneyBank,
    RetryMission,
    UseRepair,
    BuyUpgrade(String),
    BuyChassis(String),
    SelectChassis(String),
    BuyReinforcedKit,
    Save,
    Load,
    ExitGame,
}

pub struct UiContext<'a> {
    pub data: &'a GameData,
    pub session: &'a crate::state::GameSession,
    pub assets: &'a AssetManager,
    pub save_exists: bool,
    pub loaded_assets: usize,
    pub ui: &'a VirtualUi,
}

pub fn play_rect() -> Rect {
    Rect::new(0.0, PLAY_TOP, LOGICAL_WIDTH, PLAY_BOTTOM - PLAY_TOP)
}

pub fn draw_game_ui(ctx: UiContext<'_>) -> Vec<UiAction> {
    let mouse = ctx.ui.mouse_position();
    let mut actions = Vec::new();

    match ctx.session.screen {
        Screen::Title => draw_title(&ctx, mouse, &mut actions),
        Screen::MissionMap => mission_map::draw_mission_map(&ctx, mouse, &mut actions),
        Screen::Loadout => loadout::draw_loadout(&ctx, mouse, &mut actions),
        Screen::Shop => management::draw_shop(&ctx, mouse, &mut actions),
        Screen::Carriages => carriages::draw_carriages(&ctx, mouse, &mut actions),
        Screen::Guards => management::draw_guards(&ctx, mouse, &mut actions),
        Screen::Upgrades => upgrades::draw_upgrades(&ctx, mouse, &mut actions),
        Screen::Settings => management::draw_settings(&ctx, mouse, &mut actions),
        Screen::Playing => gameplay::draw_gameplay(&ctx, mouse, &mut actions),
        Screen::Paused => management::draw_pause(&ctx, mouse, &mut actions),
        Screen::Results => draw_results(&ctx, mouse, &mut actions),
        Screen::Journey => journey::draw_journey(&ctx, mouse, &mut actions),
        Screen::Outfitter => outfitter::draw_outfitter(&ctx, mouse, &mut actions),
        Screen::Records => records::draw_records(&ctx, mouse, &mut actions),
        Screen::Codex => draw_codex(&ctx, mouse, &mut actions),
    }

    // A pending confirmation is a true modal: it draws over whatever screen is
    // active and swallows the frame's interactions so no click reaches the
    // screen beneath it.
    if let Some(prompt) = ctx.session.pending_confirm {
        actions.clear();
        draw_confirm_dialog(prompt, mouse, &mut actions);
    }

    actions
}

fn draw_title(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let using_title_art = draw_title_art(ctx);
    if !using_title_art {
        draw_menu_backdrop(0.0);
        draw_crest(Rect::new(62.0, 42.0, 112.0, 106.0));
        draw_ui_text_ex(
            &ctx.data.config.display_name,
            206.0,
            88.0,
            TextStyle::new(62.0, INK).params(),
        );
        draw_ui_text_ex(
            "Escort strategy campaign",
            212.0,
            126.0,
            TextStyle::new(24.0, MUTED).params(),
        );
        draw_line(64.0, 176.0, 1074.0, 176.0, 2.0, GOLD_SOFT);
    }

    let panel = if using_title_art {
        Rect::new(72.0, 300.0, 366.0, 404.0)
    } else {
        Rect::new(86.0, 236.0, 390.0, 420.0)
    };
    draw_panel(panel, true);
    draw_section_label("Main Menu", panel.x + 26.0, panel.y + 24.0, panel.w - 52.0);

    let mut y = panel.y + 58.0;
    if virtual_button(
        Rect::new(panel.x + 26.0, y, panel.w - 52.0, 44.0),
        "New Campaign",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::RequestNewCampaign);
    }
    y += 58.0;
    if virtual_button(
        Rect::new(panel.x + 26.0, y, panel.w - 52.0, 42.0),
        "Continue",
        ctx.save_exists,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::ContinueCampaign);
    }
    y += 54.0;
    if virtual_button(
        Rect::new(panel.x + 26.0, y, panel.w - 52.0, 42.0),
        "Load Game",
        ctx.save_exists,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::Load);
    }
    y += 54.0;
    if virtual_button(
        Rect::new(panel.x + 26.0, y, panel.w - 52.0, 42.0),
        "Settings",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenSettings);
    }
    y += 54.0;
    if virtual_button(
        Rect::new(panel.x + 26.0, y, panel.w - 52.0, 42.0),
        "Field Guide",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenCodex);
    }
    y += 54.0;
    if virtual_button(
        Rect::new(panel.x + 26.0, y, panel.w - 52.0, 42.0),
        "Exit Game",
        true,
        ButtonTone::Muted,
        mouse,
    ) {
        actions.push(UiAction::ExitGame);
    }
    let _ = ctx.loaded_assets;
}

/// Field guide / bestiary: a static reference of road threats and the player's
/// own escort classes, reachable from the title menu. Reuses the in-game
/// procedural sprites so players learn to recognise both.
fn draw_codex(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    use crate::state::{CodexTab, EnemyKind, GuardKind, HazardKind};
    draw_menu_backdrop(96.0);

    let panel = Rect::new(150.0, 40.0, 980.0, 656.0);
    draw_panel(panel, true);
    draw_section_label(
        "Field Guide",
        panel.x + 34.0,
        panel.y + 30.0,
        panel.w - 68.0,
    );

    let tab = ctx.session.codex_tab;
    let tabs = [
        ("Threats", CodexTab::Threats),
        ("Guards", CodexTab::Guards),
        ("Hazards", CodexTab::Hazards),
    ];
    let tab_w = 150.0;
    let tab_gap = 14.0;
    let tabs_total = tabs.len() as f32 * tab_w + (tabs.len() as f32 - 1.0) * tab_gap;
    let tab_x = panel.x + (panel.w - tabs_total) * 0.5;
    let tab_y = panel.y + 56.0;
    for (index, (label, which)) in tabs.into_iter().enumerate() {
        let tone = if tab == which {
            ButtonTone::Positive
        } else {
            ButtonTone::Secondary
        };
        if virtual_button(
            Rect::new(tab_x + index as f32 * (tab_w + tab_gap), tab_y, tab_w, 36.0),
            label,
            true,
            tone,
            mouse,
        ) {
            actions.push(UiAction::SetCodexTab(which));
        }
    }

    let content_top = panel.y + 108.0;
    // Row height accommodates the longest tab (7 threats) without overflowing
    // the panel footer.
    let row_h = 68.0;
    match tab {
        CodexTab::Threats => {
            for (index, kind) in EnemyKind::all().into_iter().enumerate() {
                let row = codex_row_rect(panel, content_top, row_h, index);
                upgrade_visuals::draw_panel_with_fill(row, upgrade_visuals::PANEL_ALT, false);
                gameplay::draw_enemy_icon(kind, vec2(row.x + 52.0, row.y + row.h * 0.5 + 2.0));
                draw_codex_row_text(row, kind.label(), kind.threat_tag(), kind.codex_blurb());
            }
        }
        CodexTab::Guards => {
            for (index, kind) in GuardKind::all().into_iter().enumerate() {
                let row = codex_row_rect(panel, content_top, row_h, index);
                upgrade_visuals::draw_panel_with_fill(row, upgrade_visuals::PANEL_ALT, false);
                management::draw_guard_portrait(
                    vec2(row.x + 52.0, row.y + row.h * 0.5 + 4.0),
                    kind,
                    true,
                );
                let role = if kind.is_ranged() {
                    "Ranged escort"
                } else {
                    "Melee escort"
                };
                draw_codex_row_text(row, kind.label(), role, kind.description());
            }
        }
        CodexTab::Hazards => {
            for (index, kind) in HazardKind::all().into_iter().enumerate() {
                let row = codex_row_rect(panel, content_top, row_h, index);
                upgrade_visuals::draw_panel_with_fill(row, upgrade_visuals::PANEL_ALT, false);
                gameplay::draw_hazard_icon(kind, vec2(row.x + 52.0, row.y + row.h * 0.5 + 2.0));
                draw_codex_row_text(row, kind.label(), kind.effect_tag(), kind.codex_blurb());
            }
        }
    }

    if virtual_button(
        Rect::new(
            panel.x + panel.w * 0.5 - 90.0,
            panel.bottom() - 54.0,
            180.0,
            42.0,
        ),
        "Back",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::ReturnTitle);
    }
}

fn codex_row_rect(panel: Rect, top: f32, row_h: f32, index: usize) -> Rect {
    Rect::new(
        panel.x + 34.0,
        top + index as f32 * row_h,
        panel.w - 68.0,
        row_h - 10.0,
    )
}

fn draw_codex_row_text(row: Rect, label: &str, tag: &str, description: &str) {
    draw_ui_text_ex(
        label,
        row.x + 110.0,
        row.y + 28.0,
        TextStyle::new(21.0, INK).params(),
    );
    draw_badge(
        Rect::new(row.x + 110.0, row.y + 40.0, 168.0, 24.0),
        tag,
        Color::new(0.16, 0.13, 0.08, 1.0),
        UI_GOLD,
    );
    draw_ui_text_ex(
        description,
        row.x + 300.0,
        row.y + 44.0,
        TextStyle::new(15.0, MUTED).params(),
    );
}

/// Modal confirmation overlay for a staged destructive action.
fn draw_confirm_dialog(
    prompt: crate::state::ConfirmPrompt,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    use crate::state::ConfirmPrompt;

    // Body is pre-wrapped into fixed-size lines rather than fit-to-box: the
    // dynamic shrink-to-fit path rasterizes glyphs at a fractional size, which
    // can thrash the shared font atlas mid-frame.
    let (title, body, confirm_label, confirm_action) = match prompt {
        ConfirmPrompt::NewCampaign => (
            "Start New Campaign?",
            [
                "This overwrites your saved campaign.",
                "Progress on the current charter is lost for good.",
            ],
            "Overwrite Save",
            UiAction::NewCampaign,
        ),
    };

    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.62),
    );

    let dialog = Rect::new(
        (LOGICAL_WIDTH - 480.0) * 0.5,
        (LOGICAL_HEIGHT - 250.0) * 0.5,
        480.0,
        250.0,
    );
    draw_panel(dialog, true);
    draw_section_label(title, dialog.x + 30.0, dialog.y + 26.0, dialog.w - 60.0);
    let center_x = dialog.x + dialog.w * 0.5;
    let mut line_y = dialog.y + 96.0;
    for line in body {
        draw_text_centered(line, center_x, line_y, TextStyle::new(16.0, MUTED));
        line_y += 26.0;
    }

    let button_y = dialog.bottom() - 66.0;
    let button_w = (dialog.w - 60.0 - 20.0) * 0.5;
    if virtual_button(
        Rect::new(dialog.x + 30.0, button_y, button_w, 44.0),
        "Keep Save",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::DismissConfirm);
    }
    if virtual_button(
        Rect::new(dialog.x + 30.0 + button_w + 20.0, button_y, button_w, 44.0),
        confirm_label,
        true,
        ButtonTone::Danger,
        mouse,
    ) {
        actions.push(confirm_action);
    }
}

fn draw_title_art(ctx: &UiContext<'_>) -> bool {
    let Some(texture) = ctx.assets.get_texture("title_screen") else {
        return false;
    };
    draw_cover_texture(texture, Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT));
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.10),
    );
    draw_rectangle(
        0.0,
        0.0,
        500.0,
        LOGICAL_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.24),
    );
    draw_rectangle(
        0.0,
        LOGICAL_HEIGHT * 0.70,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT * 0.30,
        Color::new(0.0, 0.0, 0.0, 0.18),
    );
    true
}

fn draw_cover_texture(texture: &Texture2D, rect: Rect) {
    let texture_w = texture.width().max(1.0);
    let texture_h = texture.height().max(1.0);
    let scale = (rect.w / texture_w).max(rect.h / texture_h);
    let draw_w = texture_w * scale;
    let draw_h = texture_h * scale;
    draw_texture_ex(
        texture,
        rect.x + (rect.w - draw_w) * 0.5,
        rect.y + (rect.h - draw_h) * 0.5,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(draw_w, draw_h)),
            ..Default::default()
        },
    );
}

fn draw_results(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(110.0);
    let Some(result) = &ctx.session.result else {
        mission_map::draw_mission_map(ctx, mouse, actions);
        return;
    };

    let panel = Rect::new(320.0, 112.0, 640.0, 476.0);
    draw_panel(panel, true);

    draw_text_centered_in_box(
        if result.success {
            "Route Complete"
        } else {
            "Route Failed"
        },
        panel.x + 30.0,
        panel.y + 24.0,
        panel.w - 60.0,
        46.0,
        36.0,
        INK,
    );
    draw_text_centered_in_box(
        &result.mission_name,
        panel.x + 30.0,
        panel.y + 72.0,
        panel.w - 60.0,
        28.0,
        22.0,
        MUTED,
    );
    draw_text_centered_in_box(
        &star_label(result.stars),
        panel.x + 30.0,
        panel.y + 106.0,
        panel.w - 60.0,
        32.0,
        26.0,
        UI_GOLD,
    );

    // Bonus objective outcome: closes the loop on the goal shown at loadout.
    if let Some(met) = result.bonus_met {
        let bonus_text = ctx
            .data
            .missions
            .get(&result.mission_id)
            .map(|mission| mission.bonus_objective.as_str())
            .unwrap_or("");
        let row_y = panel.y + 158.0;
        draw_ui_text_ex(
            "Bonus",
            panel.x + 64.0,
            row_y,
            TextStyle::new(16.0, UI_GOLD).params(),
        );
        draw_ui_text_ex(
            bonus_text,
            panel.x + 132.0,
            row_y,
            TextStyle::new(14.0, MUTED).params(),
        );
        let (badge_text, badge_bg, badge_fg) = if met {
            (
                "Met",
                Color::new(0.08, 0.22, 0.12, 1.0),
                Color::new(0.64, 0.92, 0.68, 1.0),
            )
        } else {
            (
                "Missed",
                Color::new(0.24, 0.09, 0.08, 1.0),
                Color::new(0.96, 0.66, 0.60, 1.0),
            )
        };
        draw_badge(
            Rect::new(panel.right() - 150.0, row_y - 16.0, 88.0, 24.0),
            badge_text,
            badge_bg,
            badge_fg,
        );
        draw_line(
            panel.x + 64.0,
            row_y + 18.0,
            panel.right() - 64.0,
            row_y + 18.0,
            1.0,
            GOLD_SOFT,
        );
    }

    // Full-width outcome line (its value is a full sentence, too wide to column).
    let grid_top = panel.y
        + if result.bonus_met.is_some() {
            202.0
        } else {
            172.0
        };
    draw_ui_text_ex(
        "Outcome",
        panel.x + 64.0,
        grid_top,
        TextStyle::new(17.0, MUTED).params(),
    );
    draw_text_right(
        &result.reason,
        panel.right() - 64.0,
        grid_top,
        TextStyle::new(17.0, INK),
    );

    // Remaining stats in a two-column grid so nothing collides with the footer.
    let mut stats = vec![
        ("Route".to_owned(), result.route_name.clone()),
        ("Score".to_owned(), result.score.to_string()),
        ("Reward".to_owned(), format!("{} gold", result.reward)),
    ];
    if result.gold_penalty > 0 {
        stats.push((
            "Losses".to_owned(),
            format!("-{} gold", result.gold_penalty),
        ));
    }
    stats.extend([
        (
            "Carriage".to_owned(),
            format!("{:.0}%", result.carriage_health_ratio * 100.0),
        ),
        (
            "Cargo".to_owned(),
            format!("{:.0}%", result.cargo_ratio * 100.0),
        ),
    ]);
    if let (Some(label), Some(ratio)) = (&result.special_label, result.special_ratio) {
        stats.push((label.clone(), format!("{:.0}%", ratio * 100.0)));
    }
    stats.push(("Threats".to_owned(), result.enemies_defeated.to_string()));
    let time_value = result
        .time_limit
        .map(|limit| format!("{:.0}s / {:.0}s", result.elapsed, limit))
        .unwrap_or_else(|| format!("{:.0}s", result.elapsed));
    stats.push(("Time".to_owned(), time_value));

    let column_split = stats.len().div_ceil(2);
    let row_h = 30.0;
    for (index, (label, value)) in stats.iter().enumerate() {
        let (column, row) = if index < column_split {
            (0, index)
        } else {
            (1, index - column_split)
        };
        let y = grid_top + 34.0 + row as f32 * row_h;
        let (label_x, value_x) = if column == 0 {
            (panel.x + 64.0, panel.x + 300.0)
        } else {
            (panel.x + 348.0, panel.right() - 64.0)
        };
        draw_ui_text_ex(label, label_x, y, TextStyle::new(17.0, MUTED).params());
        draw_text_right(value, value_x, y, TextStyle::new(17.0, INK));
    }

    // Courier-log epilogue on a win: bookends the loadout intro. Fixed-size
    // single line (short by construction) so the capture stays atlas-safe.
    if result.success {
        if let Some(outro) = ctx
            .data
            .missions
            .get(&result.mission_id)
            .map(|mission| mission.outro_text.as_str())
            .filter(|outro| !outro.is_empty())
        {
            const COURIER_LOG: Color = Color::new(0.82, 0.71, 0.49, 0.92);
            draw_text_centered(
                outro,
                panel.x + panel.w * 0.5,
                panel.bottom() - 84.0,
                TextStyle::new(15.0, COURIER_LOG),
            );
        }
    }

    let button_y = panel.bottom() - 62.0;
    if virtual_button(
        Rect::new(panel.x + 82.0, button_y, 136.0, 40.0),
        "Map",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::OpenMap);
    }
    if virtual_button(
        Rect::new(panel.x + 252.0, button_y, 136.0, 40.0),
        "Retry",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::RetryMission);
    }
    if virtual_button(
        Rect::new(panel.x + 422.0, button_y, 136.0, 40.0),
        "Upgrades",
        true,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::OpenUpgrades);
    }
}
