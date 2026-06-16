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
fn injured_guard_recovers_after_one_mission() {
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    let report = test_report(false, vec!["swordsman".to_owned()]);

    session.apply_report(report);
    assert_eq!(
        session
            .campaign
            .guard_recovery_missions(GuardKind::Swordsman),
        1
    );

    session.apply_report(test_report(false, Vec::new()));
    assert_eq!(
        session
            .campaign
            .guard_recovery_missions(GuardKind::Swordsman),
        0
    );
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
