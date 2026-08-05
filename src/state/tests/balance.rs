//! Deterministic headless balance harness and corridor assertions.

use super::*;
use macroquad::math::{vec2, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DriverPolicy {
    Boost,
    Cruise,
    Brake,
    Mixed,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SimResult {
    pub(super) success: bool,
    pub(super) health: f32,
    pub(super) cargo: f32,
    pub(super) special: Option<f32>,
    pub(super) deadline_margin: Option<f32>,
    pub(super) enemies: u32,
    pub(super) hazards: u32,
    pub(super) reward: i64,
    pub(super) penalty: i64,
}

#[derive(Clone, Copy)]
pub(super) struct SimCase<'a> {
    pub(super) mission: &'a str,
    pub(super) route: usize,
    pub(super) difficulty: DifficultyPreset,
    pub(super) chassis: &'a str,
    pub(super) frame: &'a str,
    pub(super) policy: DriverPolicy,
    pub(super) seed: u32,
}

fn max_campaign(data: &GameData, chassis: &str, frame: &str) -> CampaignState {
    let mut campaign = CampaignState::new(&data.config, Some("muddy_road"));
    campaign.campaign_rank = 4;
    campaign.armor_level = 4;
    campaign.guard_level = 4;
    campaign.archer_level = 4;
    campaign.wheel_level = 3;
    campaign.cargo_level = 3;
    campaign.repair_level = 3;
    campaign.hubs_level = 3;
    campaign.lantern_level = 3;
    campaign.hired_guard_ids = GuardKind::all()
        .into_iter()
        .map(|kind| kind.id().to_owned())
        .collect();
    campaign.selected_guard_ids = vec![
        "shield_guard".to_owned(),
        "spearman".to_owned(),
        "swordsman".to_owned(),
    ];
    campaign.selected_ranged_ids = vec!["mage".to_owned(), "crossbow_guard".to_owned()];
    campaign.selected_equipment_ids = vec![
        "carriage_armor".to_owned(),
        "repair_kit".to_owned(),
        "warding_lantern".to_owned(),
        "reinforced_wheels".to_owned(),
    ];
    campaign.chassis_id = chassis.to_owned();
    campaign.owned_chassis_ids = vec![chassis.to_owned()];
    campaign.carriage_frame_id = frame.to_owned();
    campaign.refresh_chassis_stats(data);
    campaign.refresh_frame_stats(data);
    campaign
}

fn input(policy: DriverPolicy, run: &MissionRun) -> MissionInput {
    let closest_hazard = run
        .hazards
        .iter()
        .filter(|hazard| hazard.active && hazard.pos.y > 430.0)
        .max_by(|a, b| {
            a.pos
                .y
                .partial_cmp(&b.pos.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    let hazard_close = closest_hazard.is_some();
    let (boost, brake) = match policy {
        DriverPolicy::Boost => (true, false),
        DriverPolicy::Cruise => (false, false),
        DriverPolicy::Brake => (false, true),
        DriverPolicy::Mixed => (!hazard_close, hazard_close),
    };
    MissionInput {
        mouse: vec2(640.0, 520.0),
        pressed: false,
        down: false,
        released: false,
        repair_pressed: run.carriage.health < run.carriage.max_health * 0.35,
        play_rect: Rect::new(0.0, 0.0, 1280.0, 720.0),
        steer_left: closest_hazard.is_some_and(|hazard| hazard.pos.x >= run.carriage.pos.x),
        steer_right: closest_hazard.is_some_and(|hazard| hazard.pos.x < run.carriage.pos.x),
        boost,
        brake,
    }
}

pub(super) fn simulate(data: &GameData, case: SimCase<'_>) -> SimResult {
    let mission = data.missions.get(case.mission).unwrap();
    let mut campaign = max_campaign(data, case.chassis, case.frame);
    campaign.difficulty_preset = case.difficulty;
    if let Some(route) = mission.route_choices.get(case.route) {
        campaign.select_route_choice(mission, &route.id);
    }
    if case.seed > 0 {
        campaign.records.insert(
            mission.id.clone(),
            MissionRecord {
                best_stars: 1,
                best_score: 0,
                best_reward: 0,
                completions: case.seed,
            },
        );
    }
    let mut run = MissionRun::new(mission, &campaign);
    let report = (0..4_000)
        .find_map(|_| {
            run.handle_input(input(case.policy, &run));
            run.update(mission, 0.05)
        })
        .expect("headless route must terminate");
    SimResult {
        success: report.success,
        health: report.carriage_health_ratio,
        cargo: report.cargo_ratio,
        special: report.special_ratio,
        deadline_margin: report.time_limit.map(|limit| limit - report.elapsed),
        enemies: report.enemies_encountered,
        hazards: report.hazards_encountered,
        reward: report.reward,
        penalty: report.gold_penalty,
    }
}

#[test]
fn balance_simulation_is_reproducible_and_reports_required_axes() {
    let data = GameData::load().unwrap();
    let case = SimCase {
        mission: "bandit_bend",
        route: 0,
        difficulty: DifficultyPreset::Standard,
        chassis: "standard_wagon",
        frame: "standard",
        policy: DriverPolicy::Mixed,
        seed: 3,
    };
    let a = simulate(&data, case);
    let b = simulate(&data, case);
    assert_eq!(a, b);
    assert!(a.enemies > 0 && a.hazards > 0);
    assert!(a.reward >= 0 && a.penalty >= 0);
}

#[test]
fn difficulty_outcomes_are_strictly_ordered_in_the_balance_matrix() {
    let data = GameData::load().unwrap();
    let outcomes = |difficulty| {
        let mut total = 0.0;
        let mut wins = 0;
        for mission in data.missions_ordered() {
            for seed in 0..3 {
                let result = simulate(
                    &data,
                    SimCase {
                        mission: &mission.id,
                        route: 0,
                        difficulty,
                        chassis: "standard_wagon",
                        frame: "standard",
                        policy: DriverPolicy::Mixed,
                        seed,
                    },
                );
                total += result.health;
                wins += usize::from(result.success);
            }
        }
        (wins, total)
    };
    let relaxed = outcomes(DifficultyPreset::Relaxed);
    let standard = outcomes(DifficultyPreset::Standard);
    let hard = outcomes(DifficultyPreset::Hard);
    eprintln!("difficulty,outcomes,relaxed={relaxed:?},standard={standard:?},hard={hard:?}");
    assert!(relaxed.0 > standard.0, "wins {relaxed:?} <= {standard:?}");
    assert!(standard.0 > hard.0, "wins {standard:?} <= {hard:?}");
    assert!(relaxed.1 > standard.1, "health {relaxed:?} <= {standard:?}");
    assert!(standard.1 > hard.1, "health {standard:?} <= {hard:?}");
}

#[test]
fn throttle_policies_have_distinct_value_and_brake_preserves_cargo() {
    let data = GameData::load().unwrap();
    let run = |policy| {
        simulate(
            &data,
            SimCase {
                mission: "muddy_road",
                route: 0,
                difficulty: DifficultyPreset::Standard,
                chassis: "standard_wagon",
                frame: "standard",
                policy,
                seed: 1,
            },
        )
    };
    let boost = run(DriverPolicy::Boost);
    let cruise = run(DriverPolicy::Cruise);
    let brake = run(DriverPolicy::Brake);
    let mixed = run(DriverPolicy::Mixed);
    eprintln!("throttle,boost={boost:?},cruise={cruise:?},brake={brake:?},mixed={mixed:?}");
    assert!(brake.cargo > boost.cargo);
    assert!(mixed.reward >= boost.reward || cruise.reward >= boost.reward);
}

#[test]
fn every_standard_timed_route_has_a_viable_chassis_frame_policy() {
    let data = GameData::load().unwrap();
    for mission in data
        .missions_ordered()
        .into_iter()
        .filter(|mission| mission.time_limit.is_some())
    {
        for chassis in data.chassis_ordered() {
            for route in 0..mission.route_choices.len().max(1) {
                let mut candidates = Vec::new();
                for frame in data.carriage_frames_ordered() {
                    for policy in [DriverPolicy::Boost, DriverPolicy::Mixed] {
                        candidates.push((
                            frame.id.clone(),
                            policy,
                            simulate(
                                &data,
                                SimCase {
                                    mission: &mission.id,
                                    route,
                                    difficulty: DifficultyPreset::Standard,
                                    chassis: &chassis.id,
                                    frame: &frame.id,
                                    policy,
                                    seed: 0,
                                },
                            ),
                        ));
                    }
                }
                assert!(
                    candidates.iter().any(|(_, _, result)| result.success),
                    "{} route {route} on {}: {candidates:?}",
                    mission.id,
                    chassis.id
                );
            }
        }
    }
}
