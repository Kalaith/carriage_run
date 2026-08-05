//! Full inspectable mission matrix built from the shared headless simulator.

use super::balance::{simulate, DriverPolicy, SimCase};
use super::*;

#[test]
fn full_mission_balance_matrix_reports_every_authored_axis() {
    let data = GameData::load().unwrap();
    let mut rows = 0;
    eprintln!(
        "mission,route,difficulty,chassis,frame,success_rate,health,cargo,special,margin,enemies,hazards,reward,penalty"
    );
    for mission in data.missions_ordered() {
        for route in 0..mission.route_choices.len().max(1) {
            for difficulty in DifficultyPreset::all() {
                for chassis in data.chassis_ordered() {
                    for frame in data.carriage_frames_ordered() {
                        let samples: Vec<_> = (0..2)
                            .map(|seed| {
                                simulate(
                                    &data,
                                    SimCase {
                                        mission: &mission.id,
                                        route,
                                        difficulty,
                                        chassis: &chassis.id,
                                        frame: &frame.id,
                                        policy: DriverPolicy::Mixed,
                                        seed,
                                    },
                                )
                            })
                            .collect();
                        let mean = |value: fn(&super::balance::SimResult) -> f32| {
                            samples.iter().map(value).sum::<f32>() / samples.len() as f32
                        };
                        let success_rate = samples.iter().filter(|result| result.success).count()
                            as f32
                            / samples.len() as f32;
                        let optional_mean =
                            |value: fn(&super::balance::SimResult) -> Option<f32>| {
                                let values: Vec<f32> = samples.iter().filter_map(value).collect();
                                (!values.is_empty())
                                    .then(|| values.iter().sum::<f32>() / values.len() as f32)
                            };
                        eprintln!(
                            "{},{},{},{},{},{:.2},{:.3},{:.3},{:?},{:?},{:.1},{:.1},{:.1},{:.1}",
                            mission.id,
                            route,
                            difficulty.id(),
                            chassis.id,
                            frame.id,
                            success_rate,
                            mean(|result| result.health),
                            mean(|result| result.cargo),
                            optional_mean(|result| result.special),
                            optional_mean(|result| result.deadline_margin),
                            mean(|result| result.enemies as f32),
                            mean(|result| result.hazards as f32),
                            mean(|result| result.reward as f32),
                            mean(|result| result.penalty as f32),
                        );
                        rows += 1;
                    }
                }
            }
        }
    }
    let expected = data
        .missions_ordered()
        .into_iter()
        .map(|mission| mission.route_choices.len().max(1))
        .sum::<usize>()
        * DifficultyPreset::all().len()
        * data.chassis_ordered().len()
        * data.carriage_frames_ordered().len();
    assert_eq!(rows, expected);
}
