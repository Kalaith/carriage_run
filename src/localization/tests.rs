use super::*;

#[test]
fn missing_keys_fall_back_to_english_and_are_reported() {
    let mut localizer = Localizer::english();
    assert_eq!(localizer.text("menu.new_campaign"), "New Campaign");
    assert_eq!(localizer.text("missing.key"), "missing.key");
    assert!(localizer.missing_keys().any(|key| key == "missing.key"));
}

#[test]
fn longer_language_fallbacks_are_declared() {
    assert!(font_fallbacks(Language::German).len() >= 2);
    assert!(font_fallbacks(Language::French).len() >= 2);
}

#[test]
fn translated_layout_has_no_unbounded_strings() {
    for language in [Language::English, Language::German, Language::French] {
        let localizer = Localizer::load(language).unwrap();
        assert!(localizer.layout_warnings().is_empty());
    }
}
