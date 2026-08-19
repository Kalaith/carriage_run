//! Generated road-hazard sprites shared by gameplay and the field guide.

use super::sprites::draw_world;
use crate::state::{Hazard, HazardKind};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;

fn hazard_id(kind: HazardKind) -> &'static str {
    match kind {
        HazardKind::Mud => "mud",
        HazardKind::FallenTree => "fallen_tree",
        HazardKind::Rocks => "rocks",
        HazardKind::FirePatch => "fire_patch",
        HazardKind::RiverFord => "river_ford",
        HazardKind::Rockslide => "rockslide",
        HazardKind::CursedFog => "cursed_fog",
        HazardKind::NightStretch => "night_stretch",
    }
}

pub(super) fn draw_hazard_icon(assets: &AssetManager, kind: HazardKind, pos: Vec2) {
    draw_world(assets, hazard_id(kind), pos, vec2(70.0, 58.0), WHITE);
}

pub(super) fn draw_hazard(assets: &AssetManager, hazard: &Hazard) {
    let tint = if hazard.active {
        WHITE
    } else {
        Color::new(0.55, 0.55, 0.55, 0.58)
    };
    let size = match hazard.kind {
        HazardKind::FallenTree | HazardKind::RiverFord => vec2(150.0, 82.0),
        HazardKind::Rockslide | HazardKind::CursedFog | HazardKind::NightStretch => {
            vec2(hazard.size.x, hazard.size.y)
        }
        _ => vec2(hazard.radius * 3.2, hazard.radius * 2.7),
    };
    draw_world(assets, hazard_id(hazard.kind), hazard.pos, size, tint);
}
