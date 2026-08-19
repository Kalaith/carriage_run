//! Generated carriage sprite and equipment overlays.

use super::sprites::draw_world;
use crate::state::{CarriageDamageState, MissionRun};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;

pub(super) fn draw_carriage(assets: &AssetManager, run: &MissionRun) {
    let visual = run.carriage_visual;
    let _ = (visual.armor_level, visual.ranged_slots);
    let id = match visual.chassis_slots {
        0..=2 => "scout_cart",
        3 => "merchant_wagon",
        _ => "heavy_wagon",
    };
    let damage_tint = match run.carriage.damage_state() {
        CarriageDamageState::Pristine => WHITE,
        CarriageDamageState::Worn => Color::new(0.96, 0.88, 0.72, 1.0),
        CarriageDamageState::Damaged => Color::new(0.94, 0.66, 0.52, 1.0),
        CarriageDamageState::Critical => Color::new(1.0, 0.42, 0.36, 1.0),
    };
    let livery = run.carriage_visual.livery_tint;
    let livery_tint = Color::new(livery[0], livery[1], livery[2], 1.0);
    let tint = if run.carriage.hit_flash.finished() {
        Color::new(
            damage_tint.r * livery_tint.r,
            damage_tint.g * livery_tint.g,
            damage_tint.b * livery_tint.b,
            1.0,
        )
    } else {
        Color::new(1.0, 0.66, 0.58, 1.0)
    };
    let bob = if run.carriage.slow_timer > 0.0 {
        0.0
    } else {
        (run.carriage.animation_time * 8.0).sin() * 2.0
    };
    let carriage_pos = run.carriage.pos + vec2(0.0, bob);
    draw_world(assets, id, carriage_pos, vec2(150.0, 126.0), tint);
    let mut x = run.carriage.pos.x - 48.0;
    for (active, upgrade) in [
        (visual.iron_plating, "carriage_armor"),
        (visual.reinforced_wheels, "reinforced_wheels"),
        (visual.cargo_straps, "cargo_straps"),
        (visual.repair_kit, "repair_kit"),
        (visual.spiked_hubs, "spiked_hubs"),
        (visual.warding_lantern, "warding_lantern"),
    ] {
        if active {
            draw_world(
                assets,
                upgrade,
                vec2(x, carriage_pos.y + 48.0),
                vec2(26.0, 26.0),
                WHITE,
            );
            x += 19.0;
        }
    }
}
