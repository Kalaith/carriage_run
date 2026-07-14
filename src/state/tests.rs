//! Shared fixtures for the state tests; behaviour lives in the submodules.

mod chassis;
mod expedition;
mod progression;
mod runs;
mod validation;

use super::*;
use crate::data::GameData;

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
