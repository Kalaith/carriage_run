//! Chassis ownership, slot counts, and legacy-save fallback.

use super::*;

#[test]
fn new_campaign_starts_on_scout_chassis() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);

    assert_eq!(session.campaign.chassis_id, "scout_cart");
    assert!(session.campaign.is_chassis_owned("scout_cart"));
    assert_eq!(session.campaign.guard_slot_count(), 2);
    assert_eq!(session.campaign.carriage_equipment_slot_count(), 2);
    assert!(session.campaign.chassis_speed_mult > 1.0);
}

#[test]
fn buying_heavy_chassis_expands_slots_and_sets_active() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);
    session.campaign.gold = 1000;

    assert!(session.buy_chassis(&data, "heavy_wagon"));
    assert!(session.campaign.is_chassis_owned("heavy_wagon"));
    assert_eq!(session.campaign.chassis_id, "heavy_wagon");
    assert_eq!(session.campaign.guard_slot_count(), 4);
    assert!(session.campaign.chassis_health_mult > 1.0);
    assert!(session.campaign.chassis_speed_mult < 1.0);

    // Switching back to the owned starter is free and restores its slots.
    assert!(session.select_chassis(&data, "scout_cart"));
    assert_eq!(session.campaign.guard_slot_count(), 2);
}

#[test]
fn cannot_buy_chassis_without_gold() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);
    session.campaign.gold = 10;

    assert!(!session.buy_chassis(&data, "heavy_wagon"));
    assert!(!session.campaign.is_chassis_owned("heavy_wagon"));
}

#[test]
fn legacy_save_without_chassis_keeps_slot_count() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    // Simulate an old save: high carriage level, no chassis recorded.
    session.campaign.carriage_level = 4;
    session.campaign.chassis_id = String::new();
    session.campaign.owned_chassis_ids.clear();
    session.campaign.chassis_slots = 0;

    session.sync_chassis(&data);

    assert_eq!(session.campaign.guard_slot_count(), 4);
    assert!(session.campaign.is_chassis_owned("scout_cart"));
}
