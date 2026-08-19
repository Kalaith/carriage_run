use super::*;

#[test]
fn embedded_data_loads() {
    let data = GameData::load().unwrap();

    assert_eq!(data.config.game_name, "carriage_run");
    assert!(data.missions.contains("muddy_road"));
    assert!(data.missions.contains("siege_supply"));
    assert!(data.upgrades.contains("carriage_armor"));
    assert!(data.upgrades.contains("spiked_hubs"));
    assert!(data.upgrades.contains("warding_lantern"));
    assert_eq!(data.missions_ordered()[0].id, "muddy_road");
    assert!(data
        .missions_ordered()
        .iter()
        .all(|mission| !mission.intro_text.is_empty() && !mission.outro_text.is_empty()));
    assert_eq!(data.default_chassis_id(), "scout_cart");
    assert_eq!(data.chassis_ordered().len(), 3);
    assert_eq!(data.chassis_for_legacy_level(4), "heavy_wagon");
    assert_eq!(data.chassis_for_legacy_level(1), "scout_cart");
    assert!(!data
        .missions
        .get("bandit_bend")
        .unwrap()
        .route_choices
        .is_empty());
    assert!(data
        .texture_manifest
        .iter()
        .any(|texture| texture.key == "title_screen"));
    assert!(data.cosmetics.contains("livery_roadwarden"));
}

#[test]
fn every_mission_has_structured_bonus_criteria() {
    let data = GameData::load().unwrap();
    assert!(data
        .missions_ordered()
        .iter()
        .all(|mission| mission.bonus.is_some()));
}

#[test]
fn bonus_criteria_evaluates_each_metric() {
    let cargo = BonusCriteria {
        metric: BonusMetric::Cargo,
        threshold: 0.85,
    };
    assert!(cargo.is_met(0.90, 0.0, None, 0, None));
    assert!(!cargo.is_met(0.80, 1.0, None, 99, None));

    let threats = BonusCriteria {
        metric: BonusMetric::Threats,
        threshold: 8.0,
    };
    assert!(threats.is_met(0.0, 0.0, None, 8, None));
    assert!(!threats.is_met(1.0, 1.0, None, 7, None));

    // Special/time metrics miss when their value is absent.
    let special = BonusCriteria {
        metric: BonusMetric::Special,
        threshold: 0.70,
    };
    assert!(special.is_met(0.0, 0.0, Some(0.71), 0, None));
    assert!(!special.is_met(1.0, 1.0, None, 0, None));

    let time = BonusCriteria {
        metric: BonusMetric::TimeRemaining,
        threshold: 12.0,
    };
    assert!(time.is_met(0.0, 0.0, None, 0, Some(13.0)));
    assert!(!time.is_met(1.0, 1.0, None, 0, Some(4.0)));
    assert!(!time.is_met(1.0, 1.0, None, 0, None));
}

#[test]
fn mission_unlock_levels_are_non_decreasing_by_order() {
    let data = GameData::load().unwrap();
    for pair in data.missions_ordered().windows(2) {
        let (prev, next) = (pair[0], pair[1]);
        assert!(
            next.unlock_level >= prev.unlock_level,
            "unlock level regresses: '{}' (order {}) L{} follows '{}' (order {}) L{}",
            next.id,
            next.order,
            next.unlock_level,
            prev.id,
            prev.order,
            prev.unlock_level,
        );
        assert!(
            next.base_reward > 0,
            "'{}' has non-positive reward",
            next.id
        );
        assert!(
            next.distance > 0.0,
            "'{}' has non-positive distance",
            next.id
        );
    }
}

#[test]
fn every_upgrade_has_a_positive_cost_and_levels() {
    let data = GameData::load().unwrap();
    for (id, upgrade) in data.upgrades.iter() {
        assert!(
            upgrade.base_cost > 0,
            "upgrade '{id}' base_cost not positive"
        );
        assert!(upgrade.max_level >= 1, "upgrade '{id}' has no levels");
    }
}

#[test]
fn chassis_cost_and_slots_rise_with_order() {
    let data = GameData::load().unwrap();
    for pair in data.chassis_ordered().windows(2) {
        let (prev, next) = (pair[0], pair[1]);
        assert!(
            next.cost >= prev.cost,
            "chassis cost regresses at '{}'",
            next.id
        );
        assert!(
            next.slots >= prev.slots,
            "chassis slots regress at '{}'",
            next.id
        );
    }
}

#[test]
fn mission_difficulty_is_non_decreasing_by_order() {
    let data = GameData::load().unwrap();
    let ordered = data.missions_ordered();
    for pair in ordered.windows(2) {
        let (prev, next) = (pair[0], pair[1]);
        assert!(
            next.difficulty >= prev.difficulty,
            "difficulty regresses: '{}' (order {}) is {} but earlier '{}' (order {}) is {}",
            next.id,
            next.order,
            next.difficulty,
            prev.id,
            prev.order,
            prev.difficulty,
        );
    }
}
