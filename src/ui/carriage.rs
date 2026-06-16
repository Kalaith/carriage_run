//! Carriage rendering and equipment visuals.

use crate::state::MissionRun;
use macroquad::prelude::*;

pub(super) fn draw_carriage(run: &MissionRun) {
    let carriage = &run.carriage;
    let visual = run.carriage_visual;
    let rect = carriage.rect();
    let body = if carriage.hit_flash > 0.0 {
        Color::new(0.96, 0.92, 0.78, 1.0)
    } else {
        body_color(visual.chassis_level)
    };

    draw_rectangle(
        rect.x + 7.0,
        rect.y + 9.0,
        rect.w,
        rect.h,
        Color::new(0.0, 0.0, 0.0, 0.22),
    );
    draw_wheels(rect, visual.reinforced_wheels);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, body);
    draw_rectangle(
        rect.x + 10.0,
        rect.y + 12.0,
        rect.w - 20.0,
        34.0,
        trim_color(visual.chassis_level),
    );
    draw_rectangle(
        rect.x + 13.0,
        rect.y + 54.0,
        rect.w - 26.0,
        28.0,
        Color::new(0.38, 0.20, 0.10, 1.0),
    );

    if visual.iron_plating {
        draw_armor_plates(rect);
    }
    if visual.cargo_straps {
        draw_cargo_straps(rect);
    }
    if visual.repair_kit {
        draw_repair_kit(rect);
    }
    if visual.chassis_level >= 3 {
        draw_roof_trim(rect, visual.chassis_level);
    }

    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        Color::new(0.16, 0.08, 0.04, 1.0),
    );

    for slot in 0..visual.ranged_slots {
        let pos = run.carriage_slot_pos(slot);
        draw_circle_lines(pos.x, pos.y, 18.0, 2.0, Color::new(0.92, 0.78, 0.36, 0.72));
        draw_circle(pos.x, pos.y, 5.0, Color::new(0.92, 0.78, 0.36, 0.48));
    }

    if carriage.slow_timer > 0.0 {
        draw_circle_lines(
            carriage.pos.x,
            carriage.pos.y,
            62.0,
            3.0,
            Color::new(0.25, 0.16, 0.08, 0.7),
        );
    }
}

fn body_color(level: u32) -> Color {
    match level {
        0 | 1 => Color::new(0.56, 0.30, 0.14, 1.0),
        2 => Color::new(0.50, 0.32, 0.18, 1.0),
        3 => Color::new(0.43, 0.36, 0.28, 1.0),
        _ => Color::new(0.50, 0.24, 0.20, 1.0),
    }
}

fn trim_color(level: u32) -> Color {
    match level {
        0 | 1 => Color::new(0.72, 0.48, 0.22, 1.0),
        2 => Color::new(0.74, 0.56, 0.28, 1.0),
        3 => Color::new(0.60, 0.62, 0.58, 1.0),
        _ => Color::new(0.86, 0.68, 0.28, 1.0),
    }
}

fn draw_wheels(rect: Rect, reinforced: bool) {
    let wheel_color = Color::new(0.18, 0.10, 0.06, 1.0);
    let rim = if reinforced {
        Color::new(0.70, 0.72, 0.68, 1.0)
    } else {
        Color::new(0.24, 0.14, 0.08, 1.0)
    };
    for (x, y, radius) in [
        (rect.x + 8.0, rect.y + 22.0, 11.0),
        (rect.right() - 8.0, rect.y + 22.0, 11.0),
        (rect.x + 8.0, rect.bottom() - 20.0, 12.0),
        (rect.right() - 8.0, rect.bottom() - 20.0, 12.0),
    ] {
        draw_circle(x, y, radius, wheel_color);
        draw_circle_lines(x, y, radius + 2.0, 2.0, rim);
    }
}

fn draw_armor_plates(rect: Rect) {
    let plate = Color::new(0.62, 0.64, 0.60, 0.96);
    draw_rectangle(rect.x + 5.0, rect.y + 9.0, 9.0, rect.h - 18.0, plate);
    draw_rectangle(rect.right() - 14.0, rect.y + 9.0, 9.0, rect.h - 18.0, plate);
    draw_rectangle(rect.x + 20.0, rect.y + 6.0, rect.w - 40.0, 7.0, plate);
    draw_rectangle(
        rect.x + 20.0,
        rect.bottom() - 13.0,
        rect.w - 40.0,
        7.0,
        plate,
    );
}

fn draw_cargo_straps(rect: Rect) {
    let strap = Color::new(0.88, 0.66, 0.24, 1.0);
    draw_rectangle(rect.x + 19.0, rect.y + 58.0, rect.w - 38.0, 5.0, strap);
    draw_rectangle(rect.x + 19.0, rect.y + 75.0, rect.w - 38.0, 5.0, strap);
}

fn draw_repair_kit(rect: Rect) {
    let kit = Rect::new(rect.right() - 31.0, rect.y + 48.0, 19.0, 17.0);
    draw_rectangle(
        kit.x,
        kit.y,
        kit.w,
        kit.h,
        Color::new(0.82, 0.86, 0.78, 1.0),
    );
    draw_rectangle(
        kit.x + 7.0,
        kit.y + 3.0,
        5.0,
        kit.h - 6.0,
        Color::new(0.65, 0.12, 0.10, 1.0),
    );
    draw_rectangle(
        kit.x + 3.0,
        kit.y + 7.0,
        kit.w - 6.0,
        4.0,
        Color::new(0.65, 0.12, 0.10, 1.0),
    );
}

fn draw_roof_trim(rect: Rect, level: u32) {
    let color = if level >= 4 {
        Color::new(0.95, 0.78, 0.30, 1.0)
    } else {
        Color::new(0.56, 0.58, 0.56, 1.0)
    };
    draw_rectangle(rect.x + 20.0, rect.y + 20.0, rect.w - 40.0, 5.0, color);
    draw_rectangle(rect.x + 20.0, rect.y + 37.0, rect.w - 40.0, 5.0, color);
}
