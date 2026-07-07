use super::*;

fn test_config() -> GameConfig {
    GameConfig {
        game_name: "carriage_run".to_owned(),
        display_name: "Carriage Run".to_owned(),
        save_slot: "campaign".to_owned(),
        version: "0.1.0".to_owned(),
        starting_gold: 120,
    }
}

fn test_upgrade() -> UpgradeDef {
    UpgradeDef {
        id: "guard_training".to_owned(),
        name: "Guard Training".to_owned(),
        description: "Sharper escort work.".to_owned(),
        base_cost: 50,
        max_level: 3,
    }
}

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

#[test]
fn mission_unlock_requires_completed_path() {
    let config = test_config();
    let mut campaign = CampaignState::new(&config, Some("muddy_road"));
    let mission = test_mission("medicine_run", &["muddy_road"], &["bandit_bend"], 1);

    assert!(!campaign.is_mission_unlocked(&mission));

    campaign
        .records
        .insert("muddy_road".to_owned(), test_record());
    assert!(!campaign.is_mission_unlocked(&mission));

    campaign
        .records
        .insert("bandit_bend".to_owned(), test_record());
    assert!(campaign.is_mission_unlocked(&mission));
}

#[test]
fn near_unlock_covers_only_one_step_away_missions() {
    let config = test_config();
    let mut campaign = CampaignState::new(&config, Some("muddy_road"));

    // One carriage level short: teased on the map.
    let level_gated = test_mission("gold_shipment", &[], &[], 2);
    assert!(!campaign.is_mission_unlocked(&level_gated));
    assert!(campaign.is_mission_near_unlock(&level_gated));

    // Two levels short: still hidden.
    let far = test_mission("siege_supply", &[], &[], 3);
    assert!(!campaign.is_mission_near_unlock(&far));

    // One prerequisite away: teased, then unlocked once completed.
    let prereq_gated = test_mission("bandit_bend", &["muddy_road"], &[], 1);
    assert!(campaign.is_mission_near_unlock(&prereq_gated));
    campaign
        .records
        .insert("muddy_road".to_owned(), test_record());
    assert!(campaign.is_mission_unlocked(&prereq_gated));
    assert!(!campaign.is_mission_near_unlock(&prereq_gated));

    // A prerequisite plus a level gap is two steps: hidden.
    let two_steps = test_mission("bonebridge_pass", &["courier_deadline"], &[], 2);
    assert!(!campaign.is_mission_near_unlock(&two_steps));
}

#[test]
fn new_campaign_starts_on_scout_chassis() {
    let data = crate::data::GameData::load().unwrap();
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
    let data = crate::data::GameData::load().unwrap();
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
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);
    session.campaign.gold = 10;

    assert!(!session.buy_chassis(&data, "heavy_wagon"));
    assert!(!session.campaign.is_chassis_owned("heavy_wagon"));
}

#[test]
fn legacy_save_without_chassis_keeps_slot_count() {
    let data = crate::data::GameData::load().unwrap();
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

fn test_report(success: bool, injured_guard_ids: Vec<String>) -> MissionReport {
    MissionReport {
        mission_id: "muddy_road".to_owned(),
        mission_name: "The Muddy Road".to_owned(),
        route_name: "Forest Road".to_owned(),
        success,
        reason: "Test".to_owned(),
        stars: if success { 1 } else { 0 },
        score: 0,
        reward: 0,
        gold_penalty: 0,
        elapsed: 0.0,
        time_limit: None,
        carriage_health_ratio: 1.0,
        cargo_ratio: 1.0,
        special_label: None,
        special_ratio: None,
        enemies_defeated: 0,
        injured_guard_ids,
    }
}

fn test_record() -> MissionRecord {
    MissionRecord {
        best_stars: 1,
        best_score: 100,
        best_reward: 10,
        completions: 1,
    }
}

fn test_mission(
    id: &str,
    prerequisite_missions: &[&str],
    unlock_any_missions: &[&str],
    unlock_level: u32,
) -> crate::data::MissionDef {
    crate::data::MissionDef {
        id: id.to_owned(),
        name: "Test Mission".to_owned(),
        order: 99,
        mission_type: "cargo_transfer".to_owned(),
        route: "Test Route".to_owned(),
        cargo: "Test Cargo".to_owned(),
        objective: "Test objective.".to_owned(),
        bonus_objective: "Test bonus.".to_owned(),
        unlock_level,
        distance: 100.0,
        difficulty: 1.0,
        base_reward: 100,
        enemy_mix: Vec::new(),
        hazard_mix: Vec::new(),
        route_choices: Vec::new(),
        prerequisite_missions: prerequisite_missions
            .iter()
            .map(|id| (*id).to_owned())
            .collect(),
        unlock_any_missions: unlock_any_missions
            .iter()
            .map(|id| (*id).to_owned())
            .collect(),
        time_limit: None,
    }
}
