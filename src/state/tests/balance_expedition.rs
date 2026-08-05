//! Economy and expedition projections beside the live balance simulator.

use super::*;
use macroquad::math::{vec2, Rect};

#[derive(Debug, Clone, PartialEq)]
struct ExpeditionMetric {
    leg: u32,
    stake: String,
    modifier: String,
    starting_relic: String,
    survived: bool,
    cash_out: i64,
    failure_payout: i64,
}

fn expedition_input(run: &MissionRun) -> MissionInput {
    let hazard = run
        .hazards
        .iter()
        .filter(|hazard| hazard.active && hazard.pos.y > 430.0)
        .max_by(|a, b| {
            a.pos
                .y
                .partial_cmp(&b.pos.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    MissionInput {
        mouse: vec2(640.0, 520.0),
        pressed: false,
        down: false,
        released: false,
        repair_pressed: run.carriage.health < run.carriage.max_health * 0.35,
        play_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
        steer_left: hazard.is_some_and(|hazard| hazard.pos.x >= run.carriage.pos.x),
        steer_right: hazard.is_some_and(|hazard| hazard.pos.x < run.carriage.pos.x),
        boost: hazard.is_none(),
        brake: hazard.is_some(),
    }
}

fn expedition_session(data: &GameData, stake: &str, relic: Option<&str>) -> GameSession {
    let mut session = GameSession::new(&data.config, Some("muddy_road"));
    session.campaign.gold = 10_000;
    session.campaign.campaign_rank = 4;
    session.campaign.armor_level = 4;
    session.campaign.guard_level = 4;
    session.campaign.archer_level = 4;
    session.campaign.repair_level = 3;
    session.campaign.lantern_level = 3;
    session.campaign.selected_stake_id = stake.to_owned();
    session.campaign.hired_guard_ids = GuardKind::all()
        .into_iter()
        .map(|kind| kind.id().to_owned())
        .collect();
    session.campaign.selected_guard_ids = vec!["shield_guard".into(), "spearman".into()];
    session.campaign.selected_ranged_ids = vec!["mage".into(), "crossbow_guard".into()];
    session.campaign.selected_equipment_ids = vec![
        "carriage_armor".into(),
        "repair_kit".into(),
        "warding_lantern".into(),
    ];
    session.campaign.chassis_id = "standard_wagon".to_owned();
    session.campaign.owned_chassis_ids = vec!["standard_wagon".to_owned()];
    if let Some(relic) = relic {
        session.campaign.expedition_unlocks = vec![relic.to_owned()];
        session.campaign.selected_starting_relic_ids = vec![relic.to_owned()];
    }
    session.sync_chassis(data);
    session
}

fn simulate_expedition(
    data: &GameData,
    seed: u64,
    stake: &str,
    relic: Option<&str>,
) -> Vec<ExpeditionMetric> {
    let mut session = expedition_session(data, stake, relic);
    assert!(session.start_journey_seeded(data, seed, true));
    let relic_label = relic.unwrap_or("none").to_owned();
    let mut rows = Vec::new();
    loop {
        let (leg, modifier) = {
            let journey = session.journey.as_ref().unwrap();
            (
                journey.leg,
                journey
                    .current_leg
                    .as_ref()
                    .map(|option| option.modifier_id.clone())
                    .unwrap_or_else(|| "base".to_owned()),
            )
        };
        let report = (0..4_000)
            .find_map(|_| {
                let input = expedition_input(session.mission.as_ref().unwrap());
                session.update_play(data, 0.05, input)
            })
            .expect("expedition leg terminates");
        let journey = session.journey.as_ref().unwrap();
        rows.push(ExpeditionMetric {
            leg,
            stake: stake.to_owned(),
            modifier,
            starting_relic: relic_label.clone(),
            survived: report.success,
            cash_out: journey.banked_gold,
            failure_payout: if report.success {
                journey.banked_gold / 2
            } else {
                journey.payout
            },
        });
        if !report.success {
            break;
        }
        assert!(session.journey_choose_reward(1, data));
        if session.journey.as_ref().unwrap().won {
            break;
        }
        if let Some(event_id) = session
            .journey
            .as_ref()
            .and_then(|journey| journey.pending_event.clone())
        {
            let free = data
                .run_events
                .get(&event_id)
                .unwrap()
                .options
                .iter()
                .position(|option| option.gold >= 0)
                .unwrap();
            assert!(session.journey_resolve_event(free, data));
        }
        assert!(session.journey_begin_leg(0, data));
    }
    rows
}

#[test]
fn seeded_expedition_survival_metrics_are_reproducible() {
    let data = GameData::load().unwrap();
    let a = simulate_expedition(&data, 0xCA77, "caravan_bond", Some("iron_barding"));
    let b = simulate_expedition(&data, 0xCA77, "caravan_bond", Some("iron_barding"));
    assert_eq!(a, b);
    assert!(!a.is_empty());
    assert!(a.iter().all(|row| row.cash_out >= row.failure_payout));
}

#[test]
fn payout_projection_covers_every_leg_stake_modifier_and_starting_relic() {
    let data = GameData::load().unwrap();
    let relics: Vec<Option<&str>> = std::iter::once(None)
        .chain(
            data.relics_ordered()
                .into_iter()
                .map(|relic| Some(relic.id.as_str())),
        )
        .collect();
    let mut rows = 0;
    for stake in data.stakes_ordered() {
        for modifier in data.leg_modifiers_ordered() {
            for relic_id in &relics {
                let relic_mult = relic_id
                    .and_then(|id| data.relics.get(id))
                    .map(|relic| relic.reward_mult)
                    .unwrap_or(1.0);
                let mut cash_out = 0;
                for leg in 1..=Journey::EXPEDITION_LENGTH {
                    let leg_gold = Journey::leg_reward(leg) as f32
                        * stake.reward_mult
                        * modifier.reward_mult
                        * relic_mult;
                    cash_out += leg_gold.round() as i64;
                    let failure = cash_out / 2;
                    assert!(cash_out >= failure && failure >= 0);
                    rows += 1;
                }
            }
        }
    }
    assert_eq!(
        rows,
        data.stakes_ordered().len()
            * data.leg_modifiers_ordered().len()
            * relics.len()
            * Journey::EXPEDITION_LENGTH as usize
    );
}

#[test]
fn campaign_first_clear_income_supports_rank_purchase_milestones() {
    let data = GameData::load().unwrap();
    let ordered = data.missions_ordered();
    for stars in 1_i64..=3 {
        let rewards: Vec<i64> = ordered
            .iter()
            .map(|mission| mission.base_reward + stars * 32 + 21)
            .collect();
        let rank_two = rewards[0];
        let rank_three: i64 = rewards[..4].iter().sum();
        let rank_four: i64 = rewards[..8].iter().sum();
        assert!(rank_two >= 70, "rank 2 supports one armor purchase");
        assert!(
            rank_three >= 160 + 120,
            "rank 3 supports a wagon and recruit"
        );
        assert!(
            rank_four >= 360 + 190,
            "rank 4 supports a heavy wagon and recruit"
        );
        eprintln!("campaign,{stars}-star,rank2={rank_two},rank3={rank_three},rank4={rank_four}");
    }
}
