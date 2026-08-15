//! Shared atlas drawing for generated 2D game art.

use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;

pub(super) fn draw_atlas_sprite(
    assets: &AssetManager,
    atlas: &str,
    columns: usize,
    rows: usize,
    index: usize,
    rect: Rect,
    tint: Color,
) {
    let Some(texture) = assets.get_texture(atlas) else {
        return;
    };
    let cell_w = texture.width() / columns as f32;
    let cell_h = texture.height() / rows as f32;
    let column = index % columns;
    let row = index / columns;
    // Image generation can paint a few antialiased pixels across an atlas-cell
    // boundary. Trim a narrow gutter so neighbouring sprites never bleed in.
    let gutter = cell_w.min(cell_h) * 0.08;
    draw_texture_ex(
        texture,
        rect.x,
        rect.y,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
            source: Some(Rect::new(
                column as f32 * cell_w + gutter,
                row as f32 * cell_h + gutter,
                cell_w - gutter * 2.0,
                cell_h - gutter * 2.0,
            )),
            ..Default::default()
        },
    );
}

pub(super) fn character_index(id: &str) -> usize {
    match id {
        "wolf" => 0,
        "alpha_wolf" => 1,
        "bandit" => 2,
        "bandit_archer" => 3,
        "skeleton" => 4,
        "necromancer" => 5,
        "armored_bandit" => 6,
        "swordsman" => 7,
        "shield_guard" => 8,
        "spearman" => 9,
        "archer" => 10,
        "crossbow_guard" => 11,
        "mage" => 12,
        "princess" => 13,
        "arrow" => 14,
        "magic_bolt" => 15,
        _ => 7,
    }
}

pub(super) fn draw_character(
    assets: &AssetManager,
    id: &str,
    center: Vec2,
    size: Vec2,
    tint: Color,
) {
    draw_atlas_sprite(
        assets,
        "characters_atlas",
        4,
        4,
        character_index(id),
        Rect::new(
            center.x - size.x * 0.5,
            center.y - size.y * 0.5,
            size.x,
            size.y,
        ),
        tint,
    );
}

pub(super) fn world_index(id: &str) -> usize {
    match id {
        "mud" => 0,
        "fallen_tree" => 1,
        "rocks" => 2,
        "fire_patch" => 3,
        "river_ford" => 4,
        "scout_cart" => 5,
        "merchant_wagon" => 6,
        "heavy_wagon" => 7,
        "carriage_armor" => 8,
        "reinforced_wheels" => 9,
        "cargo_straps" => 10,
        "repair_kit" => 11,
        "spiked_hubs" => 12,
        "warding_lantern" => 13,
        "tree" => 14,
        "bush" => 15,
        _ => 6,
    }
}

pub(super) fn draw_world(assets: &AssetManager, id: &str, center: Vec2, size: Vec2, tint: Color) {
    draw_atlas_sprite(
        assets,
        "world_atlas",
        4,
        4,
        world_index(id),
        Rect::new(
            center.x - size.x * 0.5,
            center.y - size.y * 0.5,
            size.x,
            size.y,
        ),
        tint,
    );
}
