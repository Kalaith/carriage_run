//! Immediate-mode UI entry points and menu screens for Carriage Run.

mod carriage;
mod carriages;
mod cosmetics;
mod gameplay;
mod gameplay_actors;
mod gameplay_feedback;
mod gameplay_hazards;
mod gameplay_hud;
mod gameplay_road;
mod journey;
mod loadout;
mod management;
mod mission_map;
mod mission_map_art;
mod outfitter;
mod records;
mod results;
mod settings_aux;
mod sprites;
mod upgrade_visuals;
mod upgrades;
mod widgets;

use crate::data::GameData;
use crate::localization::Localizer;
use crate::settings::RuntimeSettings;
use crate::state::{Screen, PLAY_BOTTOM, PLAY_TOP};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{draw_text_centered, draw_ui_text_ex};
use std::cell::RefCell;
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
    ConfirmBuyChassis(String),
    RequestAbandonExpedition,
    AbandonExpedition,
    ContinueCampaign,
    OpenMap,
    OpenLoadout,
    OpenShop,
    OpenCarriages,
    OpenGuards,
    OpenUpgrades,
    OpenSettings,
    OpenCodex,
    OpenCosmetics,
    OpenCredits,
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
    PurchaseGuardSpecialization(String),
    BuyCosmetic(String),
    SelectCosmetic(String),
    TreatGuard(String),
    ToggleSetting(String),
    SetDifficulty(String),
    ToggleRuntimeSetting(String),
    CycleRuntimeSetting(String),
    BeginMission,
    OpenOutfitter,
    OpenRecords,
    SelectStake(String),
    UnlockStartingRelic(String),
    ToggleStartingRelic(String),
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
    SelectFrame(String),
    BuyReinforcedKit,
    Save,
    Load,
    SelectSaveSlot(String),
    CreateSaveSlot(String),
    RenameSaveSlot(String),
    DeleteSaveSlot,
    ExitGame,
}

pub struct UiContext<'a> {
    pub data: &'a GameData,
    pub session: &'a crate::state::GameSession,
    pub assets: &'a AssetManager,
    pub save_exists: bool,
    pub loaded_assets: usize,
    pub ui: &'a VirtualUi,
    pub settings: &'a RuntimeSettings,
    pub localization: &'a Localizer,
    pub save_slots: &'a [String],
    pub active_save_slot: &'a str,
    pub tooltip: &'a RefCell<macroquad_toolkit::ui::HoverTooltip>,
    pub controller_connected: bool,
}

pub fn play_rect() -> Rect {
    Rect::new(0.0, PLAY_TOP, LOGICAL_WIDTH, PLAY_BOTTOM - PLAY_TOP)
}

pub fn touch_steer_left_rect() -> Rect {
    Rect::new(186.0, 622.0, 92.0, 64.0)
}

pub fn touch_steer_right_rect() -> Rect {
    Rect::new(286.0, 622.0, 92.0, 64.0)
}

pub fn touch_brake_rect() -> Rect {
    Rect::new(946.0, 622.0, 106.0, 64.0)
}

pub fn touch_boost_rect() -> Rect {
    Rect::new(1064.0, 622.0, 106.0, 64.0)
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
        Screen::Cosmetics => cosmetics::draw_cosmetics(&ctx, mouse, &mut actions),
        Screen::Credits => draw_credits(&ctx, mouse, &mut actions),
    }

    if ctx.controller_connected {
        draw_ui_text_ex(
            "CONTROLLER  A: confirm  B: back  X: guard stance",
            30.0,
            LOGICAL_HEIGHT - 12.0,
            TextStyle::new(12.0, MUTED).params(),
        );
    }

    // A pending confirmation is a true modal: it draws over whatever screen is
    // active and swallows the frame's interactions so no click reaches the
    // screen beneath it.
    if let Some(prompt) = ctx.session.pending_confirm.clone() {
        actions.clear();
        draw_confirm_dialog(prompt, mouse, &mut actions);
    }

    actions
}

pub fn draw_recovery_screen(error: &str) {
    draw_menu_backdrop(0.0);
    let panel = Rect::new(210.0, 150.0, 860.0, 410.0);
    draw_panel(panel, true);
    draw_section_label(
        "Caravan Recovery",
        panel.x + 36.0,
        panel.y + 36.0,
        panel.w - 72.0,
    );
    draw_text_block(
        "The roadbook could not be opened. Your existing save was left untouched. Close and verify the installed data, then try again.",
        panel.x + 48.0,
        panel.y + 112.0,
        panel.w - 96.0,
        86.0,
        22.0,
        4.0,
        INK,
    );
    draw_text_block(
        error,
        panel.x + 48.0,
        panel.y + 230.0,
        panel.w - 96.0,
        80.0,
        16.0,
        4.0,
        MUTED,
    );
    draw_text_centered_in_box(
        "Native crash logging remains available for unrecoverable failures.",
        panel.x + 48.0,
        panel.bottom() - 58.0,
        panel.w - 96.0,
        24.0,
        14.0,
        GOLD_SOFT,
    );
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
        &ctx.localization.display("menu.new_campaign"),
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::RequestNewCampaign);
    }
    y += 58.0;
    if virtual_button(
        Rect::new(panel.x + 26.0, y, panel.w - 52.0, 42.0),
        &ctx.localization.display("menu.continue"),
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
        &ctx.localization.display("menu.settings"),
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenSettings);
    }
    y += 54.0;
    if virtual_button(
        Rect::new(panel.x + 26.0, y, (panel.w - 64.0) * 0.5, 42.0),
        &ctx.localization.display("menu.field_guide"),
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenCodex);
    }
    if virtual_button(
        Rect::new(
            panel.x + 34.0 + (panel.w - 64.0) * 0.5,
            y,
            (panel.w - 64.0) * 0.5,
            42.0,
        ),
        "Credits",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenCredits);
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
                gameplay_actors::draw_enemy_icon(
                    ctx.assets,
                    kind,
                    vec2(row.x + 52.0, row.y + row.h * 0.5 + 2.0),
                );
                draw_codex_row_text(row, kind.label(), kind.threat_tag(), kind.codex_blurb());
            }
        }
        CodexTab::Guards => {
            for (index, kind) in GuardKind::all().into_iter().enumerate() {
                let row = codex_row_rect(panel, content_top, row_h, index);
                upgrade_visuals::draw_panel_with_fill(row, upgrade_visuals::PANEL_ALT, false);
                management::draw_guard_portrait(
                    ctx.assets,
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
                gameplay_hazards::draw_hazard_icon(
                    ctx.assets,
                    kind,
                    vec2(row.x + 52.0, row.y + row.h * 0.5 + 2.0),
                );
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
        ConfirmPrompt::BuyChassis(id) => (
            "Buy this chassis?",
            [
                "This purchase spends campaign gold.",
                "You can switch between owned chassis later.",
            ],
            "Buy Chassis",
            UiAction::ConfirmBuyChassis(id),
        ),
        ConfirmPrompt::AbandonExpedition => (
            "Abandon Expedition?",
            [
                "Unbanked expedition gold and progress will be lost.",
                "Banking remains available as the safe return action.",
            ],
            "Abandon Run",
            UiAction::AbandonExpedition,
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
    results::draw_results(ctx, mouse, actions);
}

fn draw_credits(_ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(40.0);
    let panel = Rect::new(300.0, 86.0, 680.0, 548.0);
    draw_panel(panel, true);
    draw_section_label("Credits", panel.x + 42.0, panel.y + 34.0, panel.w - 84.0);
    draw_text_centered_in_box(
        "CARAVAN CREW",
        panel.x + 40.0,
        panel.y + 96.0,
        panel.w - 80.0,
        42.0,
        30.0,
        UI_GOLD,
    );
    draw_text_centered_in_box(
        "A small road strategy game built with Rust, macroquad, and macroquad-toolkit.",
        panel.x + 52.0,
        panel.y + 164.0,
        panel.w - 104.0,
        50.0,
        18.0,
        INK,
    );
    draw_text_centered_in_box(
        "Design, code, art direction, and testing",
        panel.x + 52.0,
        panel.y + 260.0,
        panel.w - 104.0,
        28.0,
        17.0,
        MUTED,
    );
    draw_text_centered_in_box(
        "Thanks for keeping the lantern lit.",
        panel.x + 52.0,
        panel.y + 302.0,
        panel.w - 104.0,
        28.0,
        19.0,
        UI_GOLD,
    );
    if virtual_button(
        Rect::new(panel.x + 220.0, panel.bottom() - 72.0, 240.0, 42.0),
        "Back to Title",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::ReturnTitle);
    }
}
