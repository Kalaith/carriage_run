use super::*;

#[test]
fn settings_sanitize_invalid_display_preferences() {
    let mut settings = RuntimeSettings {
        resolution: "giant".to_owned(),
        fps_cap: 2,
        text_size: 8.0,
        ..RuntimeSettings::default()
    };
    settings.sanitize();
    assert_eq!(settings.resolution, "1280x720");
    assert_eq!(settings.fps_cap, 30);
    assert_eq!(settings.text_size, 2.0);
}

#[test]
fn key_bindings_are_rebindable_without_key_literals_in_gameplay() {
    let mut bindings = KeyBindings::default();
    assert_eq!(bindings.key("boost"), KeyCode::Space);
    assert!(bindings.set("boost", "Up"));
    assert_eq!(bindings.key("boost"), KeyCode::Up);
}

#[test]
fn colorblind_palette_keeps_roles_distinct() {
    let colors = colorblind_palette(true);
    assert!(colors.windows(2).all(|pair| pair[0] != pair[1]));
}
