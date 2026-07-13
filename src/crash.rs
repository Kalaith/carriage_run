//! Native crash logging.
//!
//! Installs a panic hook that appends a formatted report to a crash log in the
//! app-data directory, so a hard failure (e.g. the startup data-load panic in
//! `game.rs`) leaves a diagnosable trace instead of vanishing. The previous
//! default hook still runs afterwards, so console output is unchanged. No-op on
//! `wasm32`, where the browser console already surfaces panics.

#[cfg(not(target_arch = "wasm32"))]
pub fn install_panic_hook(game_name: &'static str) {
    use std::io::Write;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = payload_message(info.payload());
        let location = info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));
        let report = format_report(&message, location.as_deref());

        if let Some(path) =
            macroquad_toolkit::persistence::get_app_data_path(game_name, "crash_log.txt")
        {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                let _ = writeln!(file, "{report}\n");
            }
            eprintln!("Crash report written to {}", path.display());
        }

        default_hook(info);
    }));
}

#[cfg(target_arch = "wasm32")]
pub fn install_panic_hook(_game_name: &'static str) {}

/// Best-effort extraction of a panic payload's message.
#[cfg(not(target_arch = "wasm32"))]
fn payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_owned()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "unrecognized panic payload".to_owned()
    }
}

/// Format a single crash report block. Kept free of timestamps and I/O so it is
/// deterministic and unit-testable; the hook adds surrounding context.
#[cfg(not(target_arch = "wasm32"))]
fn format_report(message: &str, location: Option<&str>) -> String {
    format!(
        "=== Carriage Run crash ===\nlocation: {}\nmessage: {}",
        location.unwrap_or("unknown"),
        message,
    )
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn report_includes_message_and_location() {
        let report = format_report("embedded data failed to load", Some("src/game.rs:35:9"));
        assert!(report.contains("embedded data failed to load"));
        assert!(report.contains("src/game.rs:35:9"));
    }

    #[test]
    fn report_marks_missing_location() {
        let report = format_report("boom", None);
        assert!(report.contains("boom"));
        assert!(report.contains("unknown"));
    }
}
