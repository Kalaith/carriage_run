//! Expedition (roguelite journey) hub and end-of-run summary.

use super::upgrade_visuals::{draw_panel, draw_section_label, GOLD as UI_GOLD, INK, MUTED};
use super::widgets::{draw_menu_backdrop, virtual_button};
use super::{UiAction, UiContext};
use crate::data::GameData;
use crate::state::{Journey, LegReward};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub(super) fn draw_journey(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    draw_menu_backdrop(64.0);
    let Some(journey) = &ctx.session.journey else {
        // No active expedition (shouldn't normally happen) — offer a way out.
        let panel = Rect::new(420.0, 240.0, 440.0, 160.0);
        draw_panel(panel, true);
        if virtual_button(
            Rect::new(panel.x + 140.0, panel.y + 90.0, 160.0, 44.0),
            "Back to Camp",
            true,
            ButtonTone::Primary,
            mouse,
        ) {
            actions.push(UiAction::OpenMap);
        }
        return;
    };

    if journey.won {
        draw_victory(journey, mouse, actions);
    } else if !journey.alive {
        draw_summary(journey, mouse, actions);
    } else if let Some(rewards) = &journey.pending_rewards {
        draw_reward_choice(journey, rewards, ctx.data, mouse, actions);
    } else if let Some(event_id) = &journey.pending_event {
        draw_run_event(event_id, ctx.data, mouse, actions);
    } else {
        draw_hub(journey, ctx.data, mouse, actions);
    }
}

/// Between-legs vignette: a short prompt and a couple of resource-trade choices.
fn draw_run_event(event_id: &str, data: &GameData, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let Some(event) = data.run_events.get(event_id) else {
        return;
    };
    let panel = Rect::new(340.0, 110.0, 600.0, 480.0);
    draw_panel(panel, true);
    draw_text_centered_in_box(
        "A Fork in the Road",
        panel.x + 30.0,
        panel.y + 30.0,
        panel.w - 60.0,
        44.0,
        32.0,
        UI_GOLD,
    );
    draw_text_block(
        &event.prompt,
        panel.x + 40.0,
        panel.y + 82.0,
        panel.w - 80.0,
        96.0,
        20.0,
        6.0,
        INK,
    );

    let mut y = panel.y + 210.0;
    for (i, option) in event.options.iter().enumerate() {
        let card = Rect::new(panel.x + 40.0, y, panel.w - 80.0, 84.0);
        draw_panel(card, false);
        draw_ui_text_ex(
            &option.label,
            card.x + 24.0,
            card.y + 32.0,
            TextStyle::new(23.0, INK).params(),
        );
        draw_ui_text_ex(
            &event_effect_summary(option),
            card.x + 24.0,
            card.y + 60.0,
            TextStyle::new(17.0, MUTED).params(),
        );
        if virtual_button(
            Rect::new(card.right() - 148.0, card.y + 22.0, 124.0, 42.0),
            "Choose",
            true,
            ButtonTone::Positive,
            mouse,
        ) {
            actions.push(UiAction::JourneyResolveEvent(i));
        }
        y += 96.0;
    }
}

/// Compact effect readout for a run-event choice ("−45 gold · +16% health").
fn event_effect_summary(option: &crate::data::RunEventOptionDef) -> String {
    let mut parts: Vec<String> = Vec::new();
    if option.gold != 0 {
        parts.push(format!("{:+} gold", option.gold));
    }
    if option.health.abs() > f32::EPSILON {
        parts.push(format!(
            "{:+}% health",
            (option.health * 100.0).round() as i32
        ));
    }
    if !option.relic.is_empty() {
        parts.push("gain a relic".to_owned());
    }
    if parts.is_empty() {
        "No cost".to_owned()
    } else {
        parts.join(" · ")
    }
}

/// Post-leg reward screen: pick one of three trades before pressing on.
fn draw_reward_choice(
    journey: &Journey,
    rewards: &[LegReward; 3],
    data: &GameData,
    mouse: Vec2,
    actions: &mut Vec<UiAction>,
) {
    let panel = Rect::new(360.0, 96.0, 560.0, 520.0);
    draw_panel(panel, true);
    draw_text_centered_in_box(
        &format!("Leg {} Cleared", journey.leg),
        panel.x + 30.0,
        panel.y + 28.0,
        panel.w - 60.0,
        44.0,
        34.0,
        INK,
    );
    draw_text_centered_in_box(
        &format!("{} — choose your spoils", journey.last_mission_name),
        panel.x + 30.0,
        panel.y + 74.0,
        panel.w - 60.0,
        26.0,
        18.0,
        MUTED,
    );

    let health_pct = (journey.carriage_health_ratio * 100.0).round() as i32;
    draw_text_centered_in_box(
        &format!("Carriage Health {}%", health_pct),
        panel.x + 30.0,
        panel.y + 110.0,
        panel.w - 60.0,
        24.0,
        17.0,
        Color::new(0.42, 0.86, 0.46, 1.0),
    );

    let mut y = panel.y + 156.0;
    for (i, reward) in rewards.iter().enumerate() {
        let card = Rect::new(panel.x + 50.0, y, panel.w - 100.0, 96.0);
        draw_panel(card, false);
        let is_relic = matches!(reward, LegReward::Relic(_));
        draw_ui_text_ex(
            &reward.title(data),
            card.x + 24.0,
            card.y + 34.0,
            TextStyle::new(24.0, if is_relic { UI_GOLD } else { INK }).params(),
        );
        draw_ui_text_ex(
            &reward.detail(data),
            card.x + 24.0,
            card.y + 64.0,
            TextStyle::new(18.0, MUTED).params(),
        );
        if virtual_button(
            Rect::new(card.right() - 150.0, card.y + 26.0, 126.0, 44.0),
            "Take",
            true,
            ButtonTone::Positive,
            mouse,
        ) {
            actions.push(UiAction::JourneyChooseReward(i));
        }
        y += 110.0;
    }
}

fn draw_hub(journey: &Journey, data: &GameData, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let panel = Rect::new(360.0, 70.0, 560.0, 600.0);
    draw_panel(panel, true);
    draw_text_centered_in_box(
        &format!(
            "Expedition — Leg {} of {}",
            journey.leg,
            Journey::EXPEDITION_LENGTH
        ),
        panel.x + 30.0,
        panel.y + 28.0,
        panel.w - 60.0,
        44.0,
        34.0,
        INK,
    );
    if journey.last_reward > 0 {
        draw_text_centered_in_box(
            &format!(
                "{} cleared: +{} gold banked",
                journey.last_mission_name, journey.last_reward
            ),
            panel.x + 30.0,
            panel.y + 74.0,
            panel.w - 60.0,
            26.0,
            18.0,
            Color::new(0.42, 0.86, 0.46, 1.0),
        );
    } else {
        draw_text_centered_in_box(
            "Load up and set out. Damage carries between legs.",
            panel.x + 30.0,
            panel.y + 74.0,
            panel.w - 60.0,
            26.0,
            18.0,
            MUTED,
        );
    }

    draw_section_label(
        "Convoy Status",
        panel.x + 40.0,
        panel.y + 128.0,
        panel.w - 80.0,
    );
    let health_pct = (journey.carriage_health_ratio * 100.0).round() as i32;
    draw_stat_line(
        panel,
        panel.y + 158.0,
        "Carriage Health",
        &format!("{}%", health_pct),
    );
    draw_stat_line(
        panel,
        panel.y + 194.0,
        "Banked Gold",
        &format!("{}", journey.banked_gold),
    );
    draw_stat_line(
        panel,
        panel.y + 230.0,
        "Next Leg Reward",
        &format!("+{}", Journey::leg_reward(journey.leg)),
    );
    let next_scale = ((journey.difficulty_scale() - 1.0) * 100.0).round() as i32;
    draw_stat_line(
        panel,
        panel.y + 266.0,
        "Threat Level",
        &format!("+{}%", next_scale),
    );
    if !journey.relics.is_empty() {
        let names: Vec<&str> = journey
            .relics
            .iter()
            .filter_map(|id| data.relics.get(id).map(|relic| relic.name.as_str()))
            .collect();
        draw_stat_line(panel, panel.y + 296.0, "Relics", &names.join(", "));
    }

    let road_label = if Journey::is_final_leg(journey.leg) {
        "Final Leg — Choose the Road Home"
    } else {
        "Choose the Next Road"
    };
    draw_section_label(road_label, panel.x + 40.0, panel.y + 322.0, panel.w - 80.0);
    let mut y = panel.y + 350.0;
    match &journey.pending_legs {
        Some(legs) if !legs.is_empty() => {
            for (i, option) in legs.iter().enumerate() {
                let route = data
                    .missions
                    .get(&option.mission_id)
                    .map(|mission| mission.route.as_str())
                    .unwrap_or("");
                if virtual_button(
                    Rect::new(panel.x + 50.0, y, panel.w - 100.0, 40.0),
                    &format!("{} — {}", option.title(data), route),
                    true,
                    ButtonTone::Positive,
                    mouse,
                ) {
                    actions.push(UiAction::JourneyBeginLeg(i));
                }
                y += 46.0;
            }
        }
        _ => {
            if virtual_button(
                Rect::new(panel.x + 50.0, y, panel.w - 100.0, 40.0),
                &format!("Press On to Leg {}", journey.leg),
                true,
                ButtonTone::Positive,
                mouse,
            ) {
                actions.push(UiAction::JourneyPressOn);
            }
            y += 46.0;
        }
    }
    y += 8.0;
    let repair_label = if journey.carriage_health_ratio >= 0.995 {
        "Carriage Fully Repaired".to_owned()
    } else {
        format!("Field Repairs ({} gold)", journey.repair_cost())
    };
    if virtual_button(
        Rect::new(panel.x + 50.0, y, panel.w - 100.0, 40.0),
        &repair_label,
        journey.can_repair(),
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::JourneyRepair);
    }
    y += 46.0;
    if virtual_button(
        Rect::new(panel.x + 50.0, y, panel.w - 100.0, 40.0),
        &format!("Bank {} Gold & Return", journey.banked_gold),
        true,
        ButtonTone::Secondary,
        mouse,
    ) {
        actions.push(UiAction::JourneyBank);
    }
}

fn draw_victory(journey: &Journey, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let panel = Rect::new(400.0, 140.0, 480.0, 380.0);
    draw_panel(panel, true);
    draw_text_centered_in_box(
        "Expedition Complete!",
        panel.x + 30.0,
        panel.y + 34.0,
        panel.w - 60.0,
        46.0,
        34.0,
        Color::new(0.42, 0.86, 0.46, 1.0),
    );
    draw_text_centered_in_box(
        &format!(
            "The convoy ran all {} legs and rolled home.",
            Journey::EXPEDITION_LENGTH
        ),
        panel.x + 30.0,
        panel.y + 88.0,
        panel.w - 60.0,
        28.0,
        19.0,
        MUTED,
    );

    draw_stat_line(
        panel,
        panel.y + 150.0,
        "Legs Cleared",
        &format!("{}", Journey::EXPEDITION_LENGTH),
    );
    draw_stat_line(
        panel,
        panel.y + 186.0,
        "Completion Bonus",
        &format!("+{} gold", journey.payout),
    );
    draw_stat_line(
        panel,
        panel.y + 222.0,
        "Total Banked",
        &format!("{}", journey.banked_gold),
    );

    if virtual_button(
        Rect::new(panel.x + 150.0, panel.bottom() - 70.0, 180.0, 46.0),
        &format!("Claim {} Gold", journey.banked_gold),
        true,
        ButtonTone::Positive,
        mouse,
    ) {
        actions.push(UiAction::JourneyBank);
    }
}

fn draw_summary(journey: &Journey, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let panel = Rect::new(400.0, 150.0, 480.0, 360.0);
    draw_panel(panel, true);
    draw_text_centered_in_box(
        "Expedition Ended",
        panel.x + 30.0,
        panel.y + 34.0,
        panel.w - 60.0,
        46.0,
        36.0,
        Color::new(1.0, 0.62, 0.42, 1.0),
    );
    draw_text_centered_in_box(
        &format!("The convoy fell on leg {}.", journey.leg),
        panel.x + 30.0,
        panel.y + 88.0,
        panel.w - 60.0,
        28.0,
        20.0,
        MUTED,
    );

    draw_stat_line(
        panel,
        panel.y + 150.0,
        "Legs Cleared",
        &format!("{}", journey.leg.saturating_sub(1)),
    );
    draw_stat_line(
        panel,
        panel.y + 186.0,
        "Banked",
        &format!("{}", journey.banked_gold),
    );
    draw_stat_line(
        panel,
        panel.y + 222.0,
        "Salvaged (half)",
        &format!("+{} gold", journey.payout),
    );

    if virtual_button(
        Rect::new(panel.x + 150.0, panel.bottom() - 70.0, 180.0, 46.0),
        "Return to Camp",
        true,
        ButtonTone::Primary,
        mouse,
    ) {
        actions.push(UiAction::JourneyBank);
    }
}

fn draw_stat_line(panel: Rect, y: f32, label: &str, value: &str) {
    draw_ui_text_ex(
        label,
        panel.x + 60.0,
        y,
        TextStyle::new(19.0, MUTED).params(),
    );
    draw_text_right(
        value,
        panel.right() - 60.0,
        y,
        TextStyle::new(19.0, UI_GOLD),
    );
}
