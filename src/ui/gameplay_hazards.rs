//! Road hazards, drawn live on the route and as static field-guide icons.

use crate::state::{Hazard, HazardKind};
use macroquad::prelude::*;

/// Draw a hazard's procedural look as a fixed-size icon for the field guide,
/// independent of a live `Hazard` instance.
pub(super) fn draw_hazard_icon(kind: HazardKind, pos: Vec2) {
    match kind {
        HazardKind::Mud => {
            let color = Color::new(0.16, 0.10, 0.07, 0.95);
            draw_circle(pos.x, pos.y, 18.0, color);
            draw_circle(pos.x + 13.0, pos.y - 4.0, 11.0, color);
            draw_circle(pos.x - 11.0, pos.y + 5.0, 10.0, color);
            draw_circle_lines(pos.x, pos.y, 18.0, 2.0, Color::new(0.42, 0.29, 0.17, 0.8));
        }
        HazardKind::FallenTree => {
            let trunk = Color::new(0.38, 0.21, 0.11, 1.0);
            draw_rectangle(pos.x - 24.0, pos.y - 7.0, 48.0, 14.0, trunk);
            draw_rectangle_lines(
                pos.x - 24.0,
                pos.y - 7.0,
                48.0,
                14.0,
                2.0,
                Color::new(0.16, 0.08, 0.04, 1.0),
            );
            draw_line(
                pos.x - 12.0,
                pos.y - 7.0,
                pos.x - 20.0,
                pos.y - 22.0,
                6.0,
                trunk,
            );
            draw_line(
                pos.x + 14.0,
                pos.y + 7.0,
                pos.x + 22.0,
                pos.y + 20.0,
                5.0,
                trunk,
            );
        }
        HazardKind::Rocks => {
            let color = Color::new(0.36, 0.35, 0.32, 1.0);
            draw_circle(pos.x - 12.0, pos.y + 6.0, 13.0, color);
            draw_circle(
                pos.x + 8.0,
                pos.y - 4.0,
                16.0,
                Color::new(0.41, 0.40, 0.37, 1.0),
            );
            draw_circle(pos.x + 18.0, pos.y + 9.0, 10.0, color);
        }
        HazardKind::FirePatch => {
            draw_circle(pos.x, pos.y, 18.0, Color::new(0.90, 0.24, 0.08, 0.95));
            draw_circle(
                pos.x - 8.0,
                pos.y + 5.0,
                11.0,
                Color::new(1.0, 0.62, 0.14, 0.95),
            );
            draw_circle(
                pos.x + 9.0,
                pos.y - 4.0,
                8.0,
                Color::new(0.96, 0.80, 0.20, 0.95),
            );
        }
        HazardKind::RiverFord => {
            draw_rectangle(
                pos.x - 22.0,
                pos.y - 14.0,
                44.0,
                28.0,
                Color::new(0.16, 0.34, 0.46, 0.92),
            );
            for i in 0..2 {
                let y = pos.y - 5.0 + i as f32 * 10.0;
                draw_line(
                    pos.x - 17.0,
                    y,
                    pos.x + 17.0,
                    y,
                    2.0,
                    Color::new(0.62, 0.80, 0.88, 0.7),
                );
            }
        }
    }
}

pub(super) fn draw_hazard(hazard: &Hazard) {
    match hazard.kind {
        HazardKind::Mud => {
            let color = if hazard.triggered {
                Color::new(0.20, 0.14, 0.09, 0.82)
            } else {
                Color::new(0.16, 0.10, 0.07, 0.92)
            };
            draw_circle(hazard.pos.x, hazard.pos.y, hazard.radius, color);
            draw_circle(
                hazard.pos.x + 24.0,
                hazard.pos.y - 5.0,
                hazard.radius * 0.62,
                color,
            );
            draw_circle(
                hazard.pos.x - 20.0,
                hazard.pos.y + 8.0,
                hazard.radius * 0.56,
                color,
            );
            draw_circle_lines(
                hazard.pos.x,
                hazard.pos.y,
                hazard.radius,
                2.0,
                Color::new(0.42, 0.29, 0.17, 0.7),
            );
        }
        HazardKind::FallenTree => {
            let rect = hazard.rect();
            let trunk = if hazard.active {
                Color::new(0.38, 0.21, 0.11, 1.0)
            } else {
                Color::new(0.25, 0.16, 0.10, 0.65)
            };
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, trunk);
            draw_rectangle_lines(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                2.0,
                Color::new(0.16, 0.08, 0.04, 1.0),
            );
            draw_line(
                rect.x + 42.0,
                rect.y,
                rect.x + 10.0,
                rect.y - 25.0,
                8.0,
                trunk,
            );
            draw_line(
                rect.right() - 52.0,
                rect.bottom(),
                rect.right() - 14.0,
                rect.bottom() + 23.0,
                7.0,
                trunk,
            );
        }
        HazardKind::Rocks => {
            let color = if hazard.active {
                Color::new(0.36, 0.35, 0.32, 1.0)
            } else {
                Color::new(0.22, 0.21, 0.20, 0.55)
            };
            draw_circle(hazard.pos.x - 14.0, hazard.pos.y + 8.0, 17.0, color);
            draw_circle(
                hazard.pos.x + 10.0,
                hazard.pos.y - 5.0,
                21.0,
                Color::new(color.r + 0.05, color.g + 0.05, color.b + 0.05, color.a),
            );
            draw_circle(hazard.pos.x + 24.0, hazard.pos.y + 12.0, 13.0, color);
        }
        HazardKind::FirePatch => {
            let alpha = if hazard.triggered { 0.72 } else { 0.92 };
            draw_circle(
                hazard.pos.x,
                hazard.pos.y,
                hazard.radius,
                Color::new(0.90, 0.24, 0.08, alpha),
            );
            draw_circle(
                hazard.pos.x - 15.0,
                hazard.pos.y + 9.0,
                hazard.radius * 0.58,
                Color::new(1.0, 0.62, 0.14, alpha),
            );
            draw_circle(
                hazard.pos.x + 20.0,
                hazard.pos.y - 7.0,
                hazard.radius * 0.46,
                Color::new(0.96, 0.80, 0.20, alpha),
            );
        }
        HazardKind::RiverFord => {
            let rect = hazard.rect();
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.16, 0.34, 0.46, 0.72),
            );
            // Ripple lines across the ford.
            for i in 0..3 {
                let y = rect.y + rect.h * (0.28 + i as f32 * 0.22);
                draw_line(
                    rect.x + 8.0,
                    y,
                    rect.right() - 8.0,
                    y,
                    2.0,
                    Color::new(0.62, 0.80, 0.88, 0.55),
                );
            }
        }
    }
}
