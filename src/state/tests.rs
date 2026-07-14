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
fn siege_run_sim_stays_bounded_and_terminates() {
    use macroquad::math::{vec2, Rect};

    let data = crate::data::GameData::load().unwrap();
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    session.campaign.difficulty_preset = DifficultyPreset::Hard;

    // Drop straight into the densest mission (bypassing unlock gates for the
    // test) and run the full sim headlessly under worst-case spawn pressure.
    let mission = data.missions.get("siege_supply").unwrap();
    session.mission = Some(MissionRun::new(mission, &session.campaign));
    session.screen = Screen::Playing;

    let input = MissionInput {
        mouse: vec2(0.0, 0.0),
        pressed: false,
        down: false,
        released: false,
        repair_pressed: false,
        play_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
        steer_left: false,
        steer_right: false,
        boost: false,
        brake: false,
    };

    let mut ended = false;
    for _ in 0..8000 {
        if session.update_play(&data, 1.0 / 60.0, input).is_some() {
            ended = true;
            break;
        }
        // Runaway spawns would blow well past the hard cap (48).
        let live = session.mission.as_ref().unwrap().enemies.len();
        assert!(live < 64, "live enemy count unbounded: {live}");
    }
    assert!(ended, "mission did not terminate within the frame budget");
}

#[test]
fn buying_and_spending_a_reinforced_kit() {
    let data = crate::data::GameData::load().unwrap();
    let config = test_config();
    let mut session = GameSession::new(&config, Some("muddy_road"));
    session.campaign.gold = crate::state::REINFORCED_KIT_COST;

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
fn generous_timers_extends_timed_missions() {
    let data = crate::data::GameData::load().unwrap();
    let mission = data.missions.get("courier_deadline").unwrap();
    let config = test_config();
    let mut campaign = CampaignState::new(&config, Some("muddy_road"));

    campaign.generous_timers = false;
    let base = MissionRun::new(mission, &campaign).time_limit.unwrap();
    campaign.generous_timers = true;
    let extended = MissionRun::new(mission, &campaign).time_limit.unwrap();

    assert!(extended > base, "timer not extended: {extended} !> {base}");
    assert!(
        (extended - base - 15.0).abs() < 0.01,
        "wrong bonus: {extended} vs {base}"
    );
}

#[test]
fn difficulty_preset_scales_mission_difficulty() {
    let config = test_config();
    let mission = test_mission("muddy_road", &[], &[], 1);
    let mut campaign = CampaignState::new(&config, Some("muddy_road"));

    campaign.difficulty_preset = DifficultyPreset::Standard;
    let standard = MissionRun::new(&mission, &campaign).difficulty;
    campaign.difficulty_preset = DifficultyPreset::Relaxed;
    let relaxed = MissionRun::new(&mission, &campaign).difficulty;
    campaign.difficulty_preset = DifficultyPreset::Hard;
    let hard = MissionRun::new(&mission, &campaign).difficulty;

    assert!((standard - 1.0).abs() < 1e-3, "standard was {standard}");
    assert!(
        relaxed < standard,
        "relaxed {relaxed} !< standard {standard}"
    );
    assert!(hard > standard, "hard {hard} !> standard {standard}");
}

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

#[test]
fn expedition_banks_rewards_and_advances_legs() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);

    assert!(session.start_journey(&data, 7));
    assert_eq!(session.journey.as_ref().unwrap().leg, 1);
    assert!(session.mission.is_some());

    // Clear leg 1: offers reward choices, holding at the same leg until picked.
    let mut report = test_report(true, Vec::new());
    report.carriage_health_ratio = 0.6;
    session.resolve_journey_leg(&report, &data);
    let journey = session.journey.as_ref().unwrap();
    assert_eq!(journey.leg, 1);
    assert!(journey.pending_rewards.is_some());
    assert!(session.mission.is_none());

    // Pressing on is blocked until a reward is chosen.
    assert!(!session.journey_press_on(&data));

    // Take War Provisions (index 1): banks base gold + a partial heal, advances.
    let base = super::Journey::leg_reward(1);
    assert!(session.journey_choose_reward(1, &data));
    let journey = session.journey.as_ref().unwrap();
    assert_eq!(journey.leg, 2);
    assert!(journey.pending_rewards.is_none());
    assert_eq!(journey.banked_gold, base);
    assert!((journey.carriage_health_ratio - 0.85).abs() < 0.001);
}

#[test]
fn expedition_relic_offer_is_collected_and_applied() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);

    assert!(session.start_journey(&data, 7));

    // Leg 1's first reward slot is a relic offer (no relics owned yet).
    session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
    let offered = match &session
        .journey
        .as_ref()
        .unwrap()
        .pending_rewards
        .as_ref()
        .unwrap()[0]
    {
        super::LegReward::Relic(id) => id.clone(),
        other => panic!("expected a relic offer, got {:?}", other),
    };

    // Taking it collects the relic (session-scoped) and banks no gold.
    assert!(session.journey_choose_reward(0, &data));
    let journey = session.journey.as_ref().unwrap();
    assert_eq!(journey.relics, vec![offered.clone()]);
    assert_eq!(journey.banked_gold, 0);

    // The relic is a real def (loads from relics.json). A between-legs vignette
    // must be resolved before the next leg can begin.
    assert!(data.relics.get(&offered).is_some());
    assert!(session.journey.as_ref().unwrap().pending_event.is_some());
    assert!(!session.journey_press_on(&data)); // blocked until the event resolves
    assert!(session.journey_resolve_event(1, &data));
    assert!(session.journey_press_on(&data));
    assert!(session.mission.is_some());
    assert!(session.journey.as_ref().unwrap().relics.contains(&offered));
}

#[test]
fn expedition_offers_bespoke_leg_branch_and_applies_modifier() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);

    assert!(session.start_journey(&data, 7));
    // Leg 1 auto-starts with no branch composition.
    assert!(session.journey.as_ref().unwrap().current_leg.is_none());

    // Clear leg 1 and take a reward: a branch of next-leg options appears.
    session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
    assert!(session.journey_choose_reward(1, &data));
    let legs = session
        .journey
        .as_ref()
        .unwrap()
        .pending_legs
        .clone()
        .expect("branch options offered");
    assert!(legs.len() >= 2, "expected a multi-option branch");
    // Options pair a real route with a distinct real modifier.
    let mut modifier_ids: Vec<&str> = legs.iter().map(|o| o.modifier_id.as_str()).collect();
    modifier_ids.sort_unstable();
    modifier_ids.dedup();
    assert_eq!(modifier_ids.len(), legs.len(), "modifiers not distinct");
    for option in &legs {
        assert!(data.missions.get(&option.mission_id).is_some());
        assert!(data.leg_modifiers.get(&option.modifier_id).is_some());
    }

    // Begin the branch whose modifier adds enemies, and confirm the leg's spawn
    // pool grew to include them.
    let idx = legs
        .iter()
        .position(|o| {
            data.leg_modifiers
                .get(&o.modifier_id)
                .is_some_and(|m| !m.enemy_add.is_empty())
        })
        .expect("at least one option adds enemies");
    let modifier = data
        .leg_modifiers
        .get(&legs[idx].modifier_id)
        .unwrap()
        .clone();
    // Clear the between-legs vignette before setting out.
    if session.journey.as_ref().unwrap().pending_event.is_some() {
        assert!(session.journey_resolve_event(1, &data));
    }
    assert!(session.journey_begin_leg(idx, &data));
    let run = session.mission.as_ref().unwrap();
    for enemy in &modifier.enemy_add {
        assert!(
            run.enemy_mix.contains(enemy),
            "leg modifier enemy {enemy} not in spawn pool"
        );
    }
    assert_eq!(
        session
            .journey
            .as_ref()
            .unwrap()
            .current_leg
            .as_ref()
            .unwrap()
            .modifier_id,
        legs[idx].modifier_id
    );
}

#[test]
fn expedition_run_event_applies_option_effects() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);

    assert!(session.start_journey(&data, 7));
    let mut report = test_report(true, Vec::new());
    report.carriage_health_ratio = 0.7;
    session.resolve_journey_leg(&report, &data);
    assert!(session.journey_choose_reward(1, &data)); // banks War Provisions gold

    // A vignette is now pending; capture the state before resolving it.
    let event_id = session
        .journey
        .as_ref()
        .unwrap()
        .pending_event
        .clone()
        .expect("a run event is offered between legs");
    let event = data.run_events.get(&event_id).expect("event id resolves");
    assert!(event.options.len() >= 2, "event should offer real choices");
    let before = session.journey.as_ref().unwrap().clone();
    let option = &event.options[0];

    assert!(session.journey_resolve_event(0, &data));
    let after = session.journey.as_ref().unwrap();
    assert!(after.pending_event.is_none(), "event cleared after choice");
    assert_eq!(after.banked_gold, (before.banked_gold + option.gold).max(0));
    if option.health.abs() > f32::EPSILON {
        let expected = (before.carriage_health_ratio + option.health).clamp(0.05, 1.0);
        assert!((after.carriage_health_ratio - expected).abs() < 1e-3);
    }
    assert!(after.last_event_result.is_some());
    // Resolving twice is a no-op (nothing pending).
    assert!(!session.journey_resolve_event(0, &data));
}

#[test]
fn expedition_records_track_bests_and_history() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);

    // Win a run: jump to the final leg, clear it, take reward, then bank.
    session.start_journey_seeded(&data, 0xABCD, true);
    assert_eq!(session.campaign.expedition_records.runs_started, 1);
    {
        let j = session.journey.as_mut().unwrap();
        j.leg = super::Journey::EXPEDITION_LENGTH;
        j.banked_gold = 400;
    }
    session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
    session.journey_choose_reward(1, &data);
    assert!(session.journey.as_ref().unwrap().won);
    let won_banked = session.journey.as_ref().unwrap().banked_gold;
    session.journey_bank_and_return();

    // The win jumped straight to the final leg, so only that one leg counts.
    let records = &session.campaign.expedition_records;
    assert_eq!(records.wins, 1);
    assert_eq!(records.best_legs, 1);
    assert_eq!(records.best_banked, won_banked);
    assert_eq!(records.total_legs, 1);
    assert_eq!(records.history.len(), 1);
    assert!(records.history[0].won);
    assert_eq!(records.history[0].seed_code, "0000ABCD");

    // A short second run appends to history (newest first) and adds to totals.
    session.start_journey(&data, 5);
    session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
    session.journey_choose_reward(1, &data);
    if session.journey.as_ref().unwrap().pending_event.is_some() {
        session.journey_resolve_event(1, &data);
    }
    session.journey_bank_and_return();

    let records = &session.campaign.expedition_records;
    assert_eq!(records.runs_started, 2);
    assert_eq!(records.history.len(), 2);
    assert!(!records.history[0].won, "newest run is first");
    assert_eq!(records.total_legs, 2);
    assert!(!records.history[0].seeded, "free run not flagged seeded");
}

#[test]
fn seeded_expeditions_are_reproducible_and_seeds_vary_runs() {
    let data = crate::data::GameData::load().unwrap();
    let config = test_config();

    // Same seed → identical next-leg branch and run event; different seed → not.
    let branch_and_event = |seed: u64| {
        let mut session = GameSession::new(&config, Some("muddy_road"));
        session.sync_chassis(&data);
        session.start_journey_seeded(&data, seed, true);
        session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
        session.journey_choose_reward(1, &data);
        let j = session.journey.as_ref().unwrap();
        let legs: Vec<String> = j
            .pending_legs
            .as_ref()
            .unwrap()
            .iter()
            .map(|o| format!("{}:{}", o.mission_id, o.modifier_id))
            .collect();
        (legs, j.pending_event.clone(), j.seed_code())
    };

    let a1 = branch_and_event(0xABCD);
    let a2 = branch_and_event(0xABCD);
    let b = branch_and_event(0x1234);
    assert_eq!(a1, a2, "same seed must reproduce the same run");
    assert_ne!(a1.0, b.0, "different seeds should diverge");
    assert_eq!(a1.2, "0000ABCD", "seed code is a stable shareable hex");
}

#[test]
fn expedition_awards_tokens_and_unlocks_persist_as_starting_relics() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);
    assert_eq!(session.campaign.expedition_tokens, 0);

    // Run two legs, then bank early: tokens = legs cleared (no win bonus).
    assert!(session.start_journey(&data, 7));
    for _ in 0..2 {
        session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
        assert!(session.journey_choose_reward(1, &data));
        if session.journey.as_ref().unwrap().pending_event.is_some() {
            session.journey_resolve_event(1, &data);
        }
    }
    session.journey_bank_and_return();
    assert_eq!(session.campaign.expedition_tokens, 2);

    // Too poor to unlock anything but the first (cost 10) once we top up.
    let relic = data.relics_ordered()[0].id.clone();
    assert!(!session.unlock_starting_relic(&relic, &data));
    session.campaign.expedition_tokens = super::Journey::STARTING_RELIC_COST + 1;
    assert!(session.unlock_starting_relic(&relic, &data));
    assert_eq!(session.campaign.expedition_tokens, 1);
    assert!(session.campaign.expedition_unlocks.contains(&relic));
    // Re-unlocking the same relic is rejected.
    assert!(!session.unlock_starting_relic(&relic, &data));

    // The next expedition now begins already holding the unlocked relic.
    assert!(session.start_journey(&data, 7));
    assert!(session.journey.as_ref().unwrap().relics.contains(&relic));
}

#[test]
fn expedition_final_leg_wins_the_run_with_a_bonus() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);
    let start_gold = session.campaign.gold;

    assert!(session.start_journey(&data, 7));
    // Jump to the final leg and bank some earnings along the way.
    {
        let journey = session.journey.as_mut().unwrap();
        journey.leg = super::Journey::EXPEDITION_LENGTH;
        journey.banked_gold = 300;
    }

    // Clear the final leg and take its reward: the run is won, not continued.
    session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
    assert!(session.journey_choose_reward(1, &data));
    let journey = session.journey.as_ref().unwrap();
    assert!(journey.won, "final leg should win the run");
    assert!(journey.alive, "a won run stays alive so it banks in full");
    assert!(journey.pending_legs.is_none(), "no branch after the finale");
    assert!(
        journey.pending_event.is_none(),
        "no vignette after the finale"
    );
    assert_eq!(journey.payout, super::Journey::completion_bonus());
    // Banked = 300 + the final leg's chosen reward + the completion bonus.
    assert!(journey.banked_gold >= 300 + super::Journey::completion_bonus());

    // Banking a won run pays out the full stash to the campaign.
    let banked = journey.banked_gold;
    session.journey_bank_and_return();
    assert!(session.journey.is_none());
    assert_eq!(session.campaign.gold, start_gold + banked);
}

#[test]
fn expedition_bank_and_return_pays_out_full() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);
    let start_gold = session.campaign.gold;

    session.start_journey(&data, 7);
    session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
    session.journey_choose_reward(1, &data); // War Provisions banks real gold
    let banked = session.journey.as_ref().unwrap().banked_gold;
    assert!(banked > 0);

    session.journey_bank_and_return();
    assert!(session.journey.is_none());
    assert_eq!(session.campaign.gold, start_gold + banked);
}

#[test]
fn expedition_failure_pays_half_and_ends_run() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);
    let start_gold = session.campaign.gold;

    session.start_journey(&data, 7);
    session.resolve_journey_leg(&test_report(true, Vec::new()), &data); // leg 1 cleared
    session.journey_choose_reward(1, &data); // bank its reward, advance to leg 2
    let banked = session.journey.as_ref().unwrap().banked_gold;

    session.resolve_journey_leg(&test_report(false, Vec::new()), &data); // leg 2 lost
    let journey = session.journey.as_ref().unwrap();
    assert!(!journey.alive);
    assert_eq!(journey.payout, banked / 2);
    assert_eq!(session.campaign.gold, start_gold + banked / 2);

    // Leaving the summary adds nothing further.
    session.journey_bank_and_return();
    assert_eq!(session.campaign.gold, start_gold + banked / 2);
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
        bonus_met: None,
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
        intro_text: String::new(),
        bonus: None,
        outro_text: String::new(),
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
