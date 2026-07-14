//! Mission content ids, unlock graph reachability, and unlock gating.

use super::*;

#[test]
fn shipped_mission_content_ids_all_resolve() {
    let data = GameData::load().unwrap();
    validate_mission_content(&data.missions_ordered()).unwrap();
}

#[test]
fn shipped_mission_graph_is_fully_reachable() {
    let data = GameData::load().unwrap();
    validate_mission_reachability(&data.missions_ordered()).unwrap();
}

#[test]
fn mission_with_unknown_prerequisite_is_rejected() {
    let mission = test_mission("orphan", &["ghost_town"], &[], 1);
    let err = validate_mission_reachability(&[&mission]).unwrap_err();
    assert!(err.contains("ghost_town"), "unknown ref not named: {err}");
    assert!(err.contains("unknown mission"), "wrong category: {err}");
}

#[test]
fn cyclic_prerequisites_are_flagged_unreachable() {
    // a needs b, b needs a: neither can ever unlock from a fresh campaign.
    let a = test_mission("a", &["b"], &[], 1);
    let b = test_mission("b", &["a"], &[], 1);
    let err = validate_mission_reachability(&[&a, &b]).unwrap_err();
    assert!(err.contains("unreachable"), "cycle not detected: {err}");
}

#[test]
fn linear_prerequisite_chain_is_reachable() {
    let start = test_mission("start", &[], &[], 1);
    let middle = test_mission("middle", &["start"], &[], 1);
    let end = test_mission("end", &["middle"], &[], 1);
    assert!(validate_mission_reachability(&[&start, &middle, &end]).is_ok());
}

#[test]
fn unknown_enemy_or_hazard_ids_are_rejected() {
    let mut mission = test_mission("bad_mission", &[], &[], 1);
    mission.enemy_mix = vec!["wolf".to_owned(), "dragon".to_owned()];
    mission.hazard_mix = vec!["lava".to_owned()];

    let err = validate_mission_content(&[&mission]).unwrap_err();
    assert!(err.contains("dragon"), "missing enemy id in error: {err}");
    assert!(err.contains("lava"), "missing hazard id in error: {err}");
}

#[test]
fn route_choice_content_ids_are_validated() {
    let mut mission = test_mission("choice_mission", &[], &[], 1);
    mission.route_choices = vec![crate::data::RouteChoiceDef {
        id: "risky_cut".to_owned(),
        name: "Risky Cut".to_owned(),
        description: "test".to_owned(),
        distance_delta: 0.0,
        difficulty_delta: 0.0,
        reward_delta: 0,
        time_limit_delta: 0.0,
        enemy_add: vec!["griffon".to_owned()],
        hazard_add: Vec::new(),
    }];

    let err = validate_mission_content(&[&mission]).unwrap_err();
    assert!(
        err.contains("choice_mission/risky_cut"),
        "route choice not located in error: {err}"
    );
    assert!(err.contains("griffon"), "missing enemy id in error: {err}");
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
