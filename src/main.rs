//! Carriage Run built with macroquad and macroquad-toolkit.

use macroquad::prelude::*;

mod data;
mod game;
mod state;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    // At logical resolution and without DPI scaling while capturing, so the
    // screenshot framebuffer is pixel-aligned with the UI layout.
    let capturing = env_string("CARRIAGE_CAPTURE_PATH").is_some();
    Conf {
        window_title: "Carriage Run".to_owned(),
        window_width: env_i32("CARRIAGE_WINDOW_WIDTH", ui::LOGICAL_WIDTH as i32),
        window_height: env_i32("CARRIAGE_WINDOW_HEIGHT", ui::LOGICAL_HEIGHT as i32),
        window_resizable: true,
        high_dpi: !capturing,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    if let Some(path) = env_string("CARRIAGE_CAPTURE_PATH") {
        run_capture(&mut game, &path).await;
        return;
    }

    loop {
        let dt = get_frame_time().min(0.1);
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}

/// Screenshot harness: boot into a scene, simulate a fixed number of steps at a
/// fixed timestep, then write a PNG and exit. Driven entirely by env vars so a
/// script can capture deterministic frames with no interactive input.
async fn run_capture(game: &mut Game, path: &str) {
    let scene = env_string("CARRIAGE_CAPTURE_SCENE").unwrap_or_else(|| "gameplay".to_owned());
    let frames = env_u32("CARRIAGE_CAPTURE_FRAMES", 150).max(1);
    let dt = 1.0 / 60.0;

    game.begin_capture_scene(&scene);
    let mut rendered = 0;
    loop {
        game.update(dt);
        game.draw();
        rendered += 1;
        // Read the framebuffer after drawing this frame but before presenting
        // it; reading after `next_frame` would return the swapped/cleared buffer.
        if rendered >= frames {
            get_screen_data().export_png(path);
            break;
        }
        next_frame().await;
    }

    println!("captured {path} (scene: {scene}, {frames} frames)");
    std::process::exit(0);
}

fn env_i32(name: &str, fallback: i32) -> i32 {
    env_string(name)
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(fallback)
}

fn env_u32(name: &str, fallback: u32) -> u32 {
    env_string(name)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(fallback)
}

fn env_string(name: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var(name).ok()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = name;
        None
    }
}
