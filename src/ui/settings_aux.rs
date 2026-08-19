//! Settings-side panels for save-slot management and display preferences.

use super::upgrade_visuals::{draw_panel, draw_section_label, GOLD, INK};
use super::widgets::virtual_button;
use super::{UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_settings_aux(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_save_slots(ctx, mouse, actions);
    draw_runtime_preferences(ctx, mouse, actions);
}

fn draw_save_slots(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let panel = Rect::new(940.0, 122.0, 300.0, 248.0);
    draw_panel(panel, true);
    draw_section_label("Save Slots", panel.x + 18.0, panel.y + 22.0, panel.w - 36.0);
    draw_ui_text_ex(
        &format!("Active: {}", ctx.active_save_slot),
        panel.x + 20.0,
        panel.y + 55.0,
        TextStyle::new(15.0, GOLD).params(),
    );
    let mut y = panel.y + 70.0;
    for slot in ctx.save_slots.iter().take(3) {
        let active = slot == ctx.active_save_slot;
        if virtual_button(
            Rect::new(panel.x + 18.0, y, panel.w - 36.0, 28.0),
            slot,
            !active,
            if active {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            mouse,
        ) {
            actions.push(UiAction::SelectSaveSlot(slot.clone()));
        }
        y += 34.0;
    }
    let next = if ctx.save_slots.iter().any(|slot| slot == "slot_2") {
        "slot_3"
    } else {
        "slot_2"
    };
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.bottom() - 46.0, 82.0, 30.0),
        "Create",
        true,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::CreateSaveSlot(next.to_owned()));
    }
    if virtual_button(
        Rect::new(panel.x + 108.0, panel.bottom() - 46.0, 82.0, 30.0),
        "Rename",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::RenameSaveSlot("archive".to_owned()));
    }
    if virtual_button(
        Rect::new(panel.x + 198.0, panel.bottom() - 46.0, 82.0, 30.0),
        "Delete",
        ctx.save_slots.len() > 1,
        ButtonTone::Danger,
        mouse,
    ) {
        actions.push(UiAction::DeleteSaveSlot);
    }
}

fn draw_runtime_preferences(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let panel = Rect::new(940.0, 386.0, 300.0, 334.0);
    draw_panel(panel, true);
    draw_section_label(
        "Display & Audio",
        panel.x + 18.0,
        panel.y + 22.0,
        panel.w - 36.0,
    );
    let settings = ctx.settings;
    let mut y = panel.y + 48.0;
    toggle_row(
        "fullscreen",
        "Fullscreen",
        settings.display.fullscreen,
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    toggle_row("vsync", "VSync", settings.vsync, panel, y, mouse, actions);
    y += 22.0;
    toggle_row(
        "colorblind_safe",
        "Colorblind palette",
        settings.colorblind_safe,
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    toggle_row(
        "reduced_motion",
        "Reduced motion",
        settings.reduced_motion,
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    toggle_row(
        "drag_toggle",
        "Drag: toggle",
        matches!(
            settings.drag_preference,
            crate::settings::DragPreference::Toggle
        ),
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "resolution",
        "Resolution",
        &settings.resolution,
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "fps",
        "FPS cap",
        &settings.fps_cap.to_string(),
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "text_size",
        "Text size",
        &format!("{:.1}x", settings.text_size),
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "master_volume",
        "Master volume",
        &percent(settings.display.master_volume),
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "sfx_volume",
        "SFX volume",
        &percent(settings.display.sfx_volume),
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "music_volume",
        "Music volume",
        &percent(settings.display.music_volume),
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "steering",
        "Steer keys",
        &format!(
            "{} / {}",
            settings.bindings.steer_left, settings.bindings.steer_right
        ),
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "recovery",
        "Repair / save / load",
        &format!(
            "{} / {} / {}",
            settings.bindings.repair, settings.bindings.save, settings.bindings.load
        ),
        panel,
        y,
        mouse,
        actions,
    );
    y += 22.0;
    cycle_row(
        "language",
        "Language",
        crate::localization::Language::from_id(&settings.language).id(),
        panel,
        y,
        mouse,
        actions,
    );
}

fn toggle_row(
    id: &str,
    label: &str,
    enabled: bool,
    panel: Rect,
    y: f32,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_ui_text_ex(
        label,
        panel.x + 18.0,
        y + 16.0,
        TextStyle::new(13.0, INK).params(),
    );
    if virtual_button(
        Rect::new(panel.right() - 92.0, y, 74.0, 21.0),
        if enabled { "On" } else { "Off" },
        true,
        if enabled {
            ButtonTone::Positive
        } else {
            ButtonTone::Secondary
        },
        mouse,
    ) {
        actions.push(UiAction::ToggleRuntimeSetting(id.to_owned()));
    }
}

fn cycle_row(
    id: &str,
    label: &str,
    value: &str,
    panel: Rect,
    y: f32,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    draw_ui_text_ex(
        label,
        panel.x + 18.0,
        y + 16.0,
        TextStyle::new(13.0, INK).params(),
    );
    if virtual_button(
        Rect::new(panel.right() - 112.0, y, 94.0, 21.0),
        value,
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::CycleRuntimeSetting(id.to_owned()));
    }
}

fn percent(value: f32) -> String {
    format!("{}%", (value * 100.0).round() as i32)
}
