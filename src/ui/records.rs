//! Expedition Records: lifetime bests and a recent-run history log with
//! per-run seeds. Reached from the Outfitter.

use super::upgrade_visuals::{draw_panel, draw_section_label, GOLD as UI_GOLD, INK, MUTED};
use super::widgets::{draw_menu_backdrop, virtual_button};
use super::{UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_records(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(64.0);
    let records = &ctx.session.campaign.expedition_records;
    let panel = Rect::new(300.0, 56.0, 680.0, 620.0);
    draw_panel(panel, true);

    draw_text_centered_in_box(
        "Expedition Records",
        panel.x + 30.0,
        panel.y + 26.0,
        panel.w - 60.0,
        44.0,
        34.0,
        INK,
    );

    // Lifetime bests.
    draw_section_label("Lifetime", panel.x + 36.0, panel.y + 82.0, panel.w - 72.0);
    let stats = [
        ("Expeditions Run", format!("{}", records.runs_started)),
        ("Completions", format!("{}", records.wins)),
        ("Best Legs Cleared", format!("{}", records.best_legs)),
        ("Best Banked", format!("{}", records.best_banked)),
        ("Total Legs Cleared", format!("{}", records.total_legs)),
    ];
    let mut y = panel.y + 116.0;
    for (label, value) in &stats {
        draw_ui_text_ex(
            label,
            panel.x + 56.0,
            y,
            TextStyle::new(19.0, MUTED).params(),
        );
        draw_text_right(
            value,
            panel.right() - 56.0,
            y,
            TextStyle::new(19.0, UI_GOLD),
        );
        y += 30.0;
    }

    // Recent-run history.
    draw_section_label(
        "Recent Runs",
        panel.x + 36.0,
        panel.y + 288.0,
        panel.w - 72.0,
    );
    if records.history.is_empty() {
        draw_text_centered_in_box(
            "No expeditions yet — set out from the Outfitter.",
            panel.x + 30.0,
            panel.y + 330.0,
            panel.w - 60.0,
            24.0,
            18.0,
            MUTED,
        );
    } else {
        let mut ry = panel.y + 320.0;
        for run in &records.history {
            let row = Rect::new(panel.x + 36.0, ry, panel.w - 72.0, 34.0);
            let outcome = if run.won {
                ("Complete", Color::new(0.42, 0.86, 0.46, 1.0))
            } else {
                ("Fell", Color::new(1.0, 0.62, 0.42, 1.0))
            };
            draw_ui_text_ex(
                &format!(
                    "{} · {} leg{}",
                    if run.seeded {
                        format!("Daily {}", run.seed_code)
                    } else {
                        "Free run".to_owned()
                    },
                    run.legs_cleared,
                    if run.legs_cleared == 1 { "" } else { "s" }
                ),
                row.x + 8.0,
                row.y + 22.0,
                TextStyle::new(18.0, INK).params(),
            );
            draw_text_right(
                &format!("{}  ·  {} gold", outcome.0, run.banked),
                row.right() - 8.0,
                row.y + 22.0,
                TextStyle::new(18.0, outcome.1),
            );
            ry += 36.0;
        }
    }

    if virtual_button(
        Rect::new(panel.x + 250.0, panel.bottom() - 58.0, 180.0, 44.0),
        "Back",
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::OpenOutfitter);
    }
}
