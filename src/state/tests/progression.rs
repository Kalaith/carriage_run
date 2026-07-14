//! Upgrades, guard roster, settings, and save migration.

use super::*;

#[test]
fn upgrade_cost_scales_from_current_level() {
    let campaign = CampaignState::new(&test_config(), Some("muddy_road"));
    let upgrade = test_upgrade();

    assert_eq!(campaign.upgrade_cost(&upgrade), Some(100));
}

#[test]
fn buying_upgrade_spends_gold_and_increases_level() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    let upgrade = test_upgrade();

    assert!(session.buy_upgrade(&upgrade));
    assert_eq!(session.campaign.gold, 20);
    assert_eq!(session.campaign.guard_level, 2);
}

#[test]
fn hiring_guard_spends_gold_and_selects_recruit() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    session.campaign.carriage_level = 2;

    assert!(session.hire_guard("shield_guard"));
    assert_eq!(session.campaign.gold, 0);
    assert!(session.campaign.is_guard_hired(GuardKind::ShieldGuard));
    assert!(session
        .campaign
        .selected_guard_ids
        .iter()
        .any(|id| id == "shield_guard"));
}

#[test]
fn upgrading_guard_star_spends_gold() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));

    assert!(session.upgrade_guard_star("archer"));
    assert_eq!(session.campaign.gold, 30);
    assert_eq!(session.campaign.guard_star_level(GuardKind::Archer), 2);
}

#[test]
fn set_difficulty_changes_preset_and_reports_change() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));

    assert_eq!(
        session.campaign.difficulty_preset,
        DifficultyPreset::Standard
    );
    assert!(session.set_difficulty("hard"));
    assert_eq!(session.campaign.difficulty_preset, DifficultyPreset::Hard);
    // Re-selecting the active preset is a no-op.
    assert!(!session.set_difficulty("hard"));
    assert!(session.set_difficulty("relaxed"));
    assert_eq!(
        session.campaign.difficulty_preset,
        DifficultyPreset::Relaxed
    );
}

#[test]
fn buying_and_spending_a_reinforced_kit() {
    let data = GameData::load().unwrap();
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    session.campaign.gold = REINFORCED_KIT_COST;

    assert!(session.buy_reinforced_kit());
    assert_eq!(session.campaign.gold, 0);
    assert_eq!(session.campaign.reinforced_kits, 1);
    // Too poor to buy another.
    assert!(!session.buy_reinforced_kit());

    // Starting a route spends the kit and the carriage sets out with the boost.
    let plain = MissionRun::new(data.missions.get("muddy_road").unwrap(), &session.campaign)
        .carriage
        .max_health;
    assert!(session.start_selected_mission(&data));
    assert_eq!(session.campaign.reinforced_kits, 0);
    let boosted = session.mission.as_ref().unwrap().carriage.max_health;
    assert!(
        boosted > plain,
        "kit boost not applied: {boosted} vs {plain}"
    );
}

#[test]
fn new_campaign_prompts_before_overwriting_an_existing_save() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));

    // A save exists: staging must not proceed, and a prompt is raised instead.
    assert!(!session.request_new_campaign(true));
    assert_eq!(session.pending_confirm, Some(ConfirmPrompt::NewCampaign));

    // Cancelling clears the prompt without touching campaign state.
    session.cancel_confirm();
    assert_eq!(session.pending_confirm, None);
}

#[test]
fn new_campaign_proceeds_immediately_when_no_save_exists() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));

    assert!(session.request_new_campaign(false));
    assert_eq!(session.pending_confirm, None);
}

#[test]
fn injured_guard_is_benched_and_recovers_over_missions() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));

    // A failed run leaves the guard worse off (3 missions).
    session.apply_report(test_report(false, vec!["swordsman".to_owned()]));
    assert_eq!(
        session
            .campaign
            .guard_recovery_missions(GuardKind::Swordsman),
        3
    );

    session.apply_report(test_report(true, Vec::new()));
    assert_eq!(
        session
            .campaign
            .guard_recovery_missions(GuardKind::Swordsman),
        2
    );
}

#[test]
fn failed_run_deducts_repair_penalty_without_going_negative() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    let start_gold = session.campaign.gold;

    let mut report = test_report(false, Vec::new());
    report.reward = 5;
    report.gold_penalty = 40;
    session.apply_report(report);
    assert_eq!(session.campaign.gold, start_gold + 5 - 40);

    // A ruinous penalty cannot push gold below zero.
    session.campaign.gold = 10;
    let mut report = test_report(false, Vec::new());
    report.gold_penalty = 500;
    session.apply_report(report);
    assert_eq!(session.campaign.gold, 0);
}

#[test]
fn treating_injured_guard_costs_gold_and_clears_recovery() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    session.apply_report(test_report(false, vec!["swordsman".to_owned()]));
    session.campaign.gold = 500;

    let cost = session
        .campaign
        .guard_treat_cost(GuardKind::Swordsman)
        .unwrap();
    assert!(session.treat_guard("swordsman"));
    assert_eq!(session.campaign.gold, 500 - cost);
    assert_eq!(
        session
            .campaign
            .guard_recovery_missions(GuardKind::Swordsman),
        0
    );
    // Nothing to treat once healed.
    assert!(!session.treat_guard("swordsman"));
}

#[test]
fn equipment_slot_assignment_requires_installed_upgrade() {
    let config = test_config();
    let mut campaign = CampaignState::new(&config, Some("muddy_road"));

    campaign.assign_equipment_slot(1, "reinforced_wheels");
    assert!(!campaign.is_equipment_equipped(CarriageEquipment::ReinforcedWheels));

    campaign.wheel_level = 1;
    campaign.assign_equipment_slot(1, "reinforced_wheels");
    assert!(campaign.is_equipment_equipped(CarriageEquipment::ReinforcedWheels));
}

#[test]
fn legacy_points_migrate_to_gold() {
    let value = serde_json::json!({ "points": 42 });
    let migrated = migrate_save_value(
        Some("1.0.0".to_owned()),
        value,
        &test_config(),
        Some("muddy_road"),
    )
    .unwrap();

    assert_eq!(migrated.version, "0.1.0");
    assert_eq!(migrated.campaign.gold, 42);
    assert_eq!(migrated.campaign.selected_mission_id, "muddy_road");
}
