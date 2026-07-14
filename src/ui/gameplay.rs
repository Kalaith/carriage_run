//! Active mission rendering: composes the road, actors, and effect layers.

use super::gameplay_actors::{draw_enemy, draw_guard, draw_shot};
use super::gameplay_hazards::draw_hazard;
use super::gameplay_hud::draw_gameplay_hud;
use super::gameplay_road::{draw_road, draw_wheel_dust};
use super::{carriage, UiAction, UiContext};
use crate::state::{DragState, MissionRun};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::TextStyle;
use macroquad_toolkit::ui::draw_text_centered;

pub(super) fn draw_gameplay(ctx: &UiContext<'_>, mouse: Vec2, actions: &mut Vec<UiAction>) {
    let Some(run) = &ctx.session.mission else {
        return;
    };

    draw_road(run, ctx.session.campaign.route_motion_enabled);
    for hazard in &run.hazards {
        draw_hazard(hazard);
    }
    for shot in &run.shots {
        draw_shot(shot);
    }
    for enemy in &run.enemies {
        draw_enemy(enemy);
    }
    for guard in run
        .guards
        .iter()
        .filter(|guard| guard.mounted_slot.is_none())
    {
        draw_guard(guard);
    }
    if ctx.session.campaign.route_motion_enabled {
        draw_wheel_dust(run);
    }
    carriage::draw_carriage(run);
    for guard in run
        .guards
        .iter()
        .filter(|guard| guard.mounted_slot.is_some())
    {
        draw_guard(guard);
    }
    draw_particles(run);
    draw_float_texts(run);
    draw_drag_feedback(run, mouse);
    draw_gameplay_hud(ctx, run, mouse, actions);
}

/// Burst particles: death scatter and combat sparks, fading and shrinking.
fn draw_particles(run: &MissionRun) {
    for particle in run.particles.particles() {
        let fade = particle.life_fraction();
        let mut color = particle.color;
        color.a = fade;
        // Shrinks to 45% of its spawn size over its life, matching the
        // original hand-rolled burst particle's falloff.
        let radius = particle.size * (0.45 + 0.55 * fade);
        draw_circle(particle.position.x, particle.position.y, radius, color);
    }
}

/// Floating combat numbers: damage dealt (gold) and taken (red), drifting up
/// and fading.
fn draw_float_texts(run: &MissionRun) {
    for text in run.float_texts.texts() {
        let mut color = text.color;
        color.a = text.life_fraction();
        draw_text_centered(
            &text.text,
            text.position.x,
            text.position.y,
            TextStyle::new(18.0, color),
        );
    }
}

fn draw_drag_feedback(run: &MissionRun, mouse: Vec2) {
    match run.drag {
        DragState::Carriage => {
            draw_line(
                run.carriage.pos.x,
                run.carriage.pos.y,
                mouse.x,
                run.carriage.pos.y - 62.0,
                3.0,
                Color::new(0.95, 0.78, 0.28, 0.85),
            );
            draw_triangle(
                vec2(mouse.x, run.carriage.pos.y - 82.0),
                vec2(mouse.x - 12.0, run.carriage.pos.y - 60.0),
                vec2(mouse.x + 12.0, run.carriage.pos.y - 60.0),
                Color::new(0.95, 0.78, 0.28, 0.85),
            );
        }
        DragState::Guard { guard_id, .. } => {
            if let Some(guard) = run.guards.iter().find(|guard| guard.id == guard_id) {
                draw_line(
                    guard.pos.x,
                    guard.pos.y,
                    mouse.x,
                    mouse.y,
                    3.0,
                    Color::new(0.38, 0.78, 0.98, 0.88),
                );
                draw_circle_lines(
                    mouse.x,
                    mouse.y,
                    26.0,
                    2.0,
                    Color::new(0.38, 0.78, 0.98, 0.88),
                );
                if guard.kind.is_ranged() {
                    for slot in 0..run.ranged_slots {
                        let pos = run.carriage_slot_pos(slot);
                        draw_circle_lines(
                            pos.x,
                            pos.y,
                            24.0,
                            3.0,
                            Color::new(0.95, 0.78, 0.28, 0.88),
                        );
                    }
                }
            }
        }
        DragState::None => {}
    }
}
