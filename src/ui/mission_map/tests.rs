use super::*;

#[test]
fn prepare_loadout_button_stays_inside_published_embed_panel() {
    assert_prepare_loadout_is_in_panel(selected_route_layout(1200.0, 675.0), 1200.0);
}

#[test]
fn prepare_loadout_button_stays_inside_standard_panel() {
    assert_prepare_loadout_is_in_panel(selected_route_layout(1280.0, 720.0), 1280.0);
}

fn assert_prepare_loadout_is_in_panel(layout: SelectedRouteLayout, screen_w: f32) {
    assert!(layout.panel.x >= 0.0);
    assert!(layout.panel.right() <= screen_w);
    assert!(layout.cta.x > layout.cta_divider);
    assert!(layout.cta.x >= layout.panel.x);
    assert!(layout.cta.right() <= layout.panel.right());
    assert!(layout.cta.bottom() <= layout.panel.bottom());
    assert!(layout.hazards.right() < layout.cta_divider);
    assert!(layout.threats.right() < layout.hazards.x);
    assert!(layout.route_choices.right() < layout.route_divider);
}
