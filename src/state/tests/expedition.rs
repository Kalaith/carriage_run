//! Roguelite expedition runs: legs, rewards, relics, stakes, and records.

use super::*;

#[test]
fn expedition_banks_rewards_and_advances_legs() {
    let data = GameData::load().unwrap();
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
    let base = Journey::leg_reward(1);
    assert!(session.journey_choose_reward(1, &data));
    let journey = session.journey.as_ref().unwrap();
    assert_eq!(journey.leg, 2);
    assert!(journey.pending_rewards.is_none());
    assert_eq!(journey.banked_gold, base);
    assert!((journey.carriage_health_ratio - 0.85).abs() < 0.001);
}

#[test]
fn expedition_relic_offer_is_collected_and_applied() {
    let data = GameData::load().unwrap();
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
        LegReward::Relic(id) => id.clone(),
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
    let data = GameData::load().unwrap();
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
    let data = GameData::load().unwrap();
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
fn expedition_entry_stake_pays_ante_and_multiplies_rewards() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);

    // Pick a paid stake tier and give enough gold to cover the ante.
    let stake = data
        .stakes_ordered()
        .into_iter()
        .find(|s| s.cost > 0)
        .unwrap()
        .clone();
    assert!(session.select_stake(&stake.id, &data));
    assert!(!session.select_stake(&stake.id, &data)); // re-selecting is a no-op
    session.campaign.gold = 500;

    // Starting the run pays the ante up front and records the multiplier.
    assert!(session.start_journey(&data, 3));
    assert_eq!(session.campaign.gold, 500 - stake.cost);
    let j = session.journey.as_ref().unwrap();
    assert!((j.stake_mult - stake.reward_mult).abs() < 1e-4);

    // The staked multiplier is baked into the leg reward choices.
    session.resolve_journey_leg(&test_report(true, Vec::new()), &data);
    let base = Journey::leg_reward(1);
    let staked_provisions = match &session
        .journey
        .as_ref()
        .unwrap()
        .pending_rewards
        .as_ref()
        .unwrap()[1]
    {
        LegReward::Provisions { gold, .. } => *gold,
        other => panic!("expected provisions, got {:?}", other),
    };
    assert_eq!(
        staked_provisions,
        ((base as f32) * stake.reward_mult).round() as i64
    );
}

#[test]
fn expedition_records_track_bests_and_history() {
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);

    // Win a run: jump to the final leg, clear it, take reward, then bank.
    session.start_journey_seeded(&data, 0xABCD, true);
    assert_eq!(session.campaign.expedition_records.runs_started, 1);
    {
        let j = session.journey.as_mut().unwrap();
        j.leg = Journey::EXPEDITION_LENGTH;
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
    let data = GameData::load().unwrap();
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
    let data = GameData::load().unwrap();
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
    session.campaign.expedition_tokens = Journey::STARTING_RELIC_COST + 1;
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
    let data = GameData::load().unwrap();
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.sync_chassis(&data);
    let start_gold = session.campaign.gold;

    assert!(session.start_journey(&data, 7));
    // Jump to the final leg and bank some earnings along the way.
    {
        let journey = session.journey.as_mut().unwrap();
        journey.leg = Journey::EXPEDITION_LENGTH;
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
    assert_eq!(journey.payout, Journey::completion_bonus());
    // Banked = 300 + the final leg's chosen reward + the completion bonus.
    assert!(journey.banked_gold >= 300 + Journey::completion_bonus());

    // Banking a won run pays out the full stash to the campaign.
    let banked = journey.banked_gold;
    session.journey_bank_and_return();
    assert!(session.journey.is_none());
    assert_eq!(session.campaign.gold, start_gold + banked);
}

#[test]
fn expedition_bank_and_return_pays_out_full() {
    let data = GameData::load().unwrap();
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
    let data = GameData::load().unwrap();
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
