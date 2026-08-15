//! Generated carriage sprite and equipment overlays.

use super::sprites::draw_world;
use crate::state::MissionRun;
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
    let tint = if run.carriage.hit_flash.finished() {
        WHITE
    } else {
        Color::new(1.0, 0.82, 0.68, 1.0)
    };
    draw_world(assets, id, run.carriage.pos, vec2(150.0, 126.0), tint);
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
                vec2(x, run.carriage.pos.y + 48.0),
                vec2(26.0, 26.0),
                WHITE,
            );
            x += 19.0;
        }
    }
}
