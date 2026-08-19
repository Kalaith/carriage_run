//! Mission results breakdown and reward explanation.

use super::mission_map;
use super::upgrade_visuals::{draw_panel, GOLD as UI_GOLD, GOLD_SOFT, INK, MUTED};
use super::widgets::{draw_menu_backdrop, star_label, virtual_button};
use super::{UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_results(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(110.0);
    let Some(result) = &ctx.session.result else {
        mission_map::draw_mission_map(ctx, mouse, actions);
        return;
    };

    let panel = Rect::new(320.0, 82.0, 640.0, 526.0);
    draw_panel(panel, true);

    let is_campaign_finale = result.success && result.mission_id == "kings_end";
    draw_text_centered_in_box(
        if is_campaign_finale {
            "The King's Road Is Open"
        } else if result.success {
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
    if is_campaign_finale {
        draw_text_centered_in_box(
            "The charter is complete. Every road now leads home.",
            panel.x + 36.0,
            panel.y + 94.0,
            panel.w - 72.0,
            24.0,
            16.0,
            UI_GOLD,
        );
    }
    draw_text_centered_in_box(
        &star_label(result.stars),
        panel.x + 30.0,
        panel.y + 106.0,
        panel.w - 60.0,
        32.0,
        26.0,
        UI_GOLD,
    );
    draw_text_centered_in_box(
        &ctx.localization.display("results.stars"),
        panel.x + 56.0,
        panel.y + 136.0,
        panel.w - 112.0,
        18.0,
        12.0,
        MUTED,
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
        (
            "Contract + Performance".to_owned(),
            format!(
                "{} gold",
                result.reward - result.reward_breakdown.bonus_objective
            ),
        ),
        (
            "Bonus Objective".to_owned(),
            format!("+{} gold", result.reward_breakdown.bonus_objective),
        ),
        ("Total Reward".to_owned(), format!("{} gold", result.reward)),
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
    stats.push((
        "Seen".to_owned(),
        format!(
            "{} threats / {} hazards",
            result.enemies_encountered, result.hazards_encountered
        ),
    ));
    let time_value = result
        .time_limit
        .map(|limit| format!("{:.0}s / {:.0}s", result.elapsed, limit))
        .unwrap_or_else(|| format!("{:.0}s", result.elapsed));
    stats.push(("Time".to_owned(), time_value));

    draw_text_centered_in_box(
        &ctx.localization.display("results.reward"),
        panel.x + 52.0,
        panel.bottom() - 116.0,
        panel.w - 104.0,
        18.0,
        12.0,
        MUTED,
    );

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
