//! Content and unlock-graph validation for shipped mission data.

use super::{EnemyKind, HazardKind};
use crate::data::MissionDef;

/// Fail fast on content typos: every enemy/hazard id referenced by mission
/// data must resolve to a known kind. Unknown ids otherwise degrade silently
/// to Wolf/Mud at spawn time (`mission/flow.rs`), shipping the wrong encounter
/// with no error. Guarded by a unit test (CI) and a debug-build assert.
pub fn validate_mission_content(missions: &[&MissionDef]) -> Result<(), String> {
    let mut unknown = Vec::new();
    let check_enemy = |unknown: &mut Vec<String>, where_: &str, id: &str| {
        if EnemyKind::from_id(id).is_none() {
            unknown.push(format!("{where_}: enemy '{id}'"));
        }
    };
    let check_hazard = |unknown: &mut Vec<String>, where_: &str, id: &str| {
        if HazardKind::from_id(id).is_none() {
            unknown.push(format!("{where_}: hazard '{id}'"));
        }
    };

    for mission in missions {
        for id in &mission.enemy_mix {
            check_enemy(&mut unknown, &mission.id, id);
        }
        for id in &mission.hazard_mix {
            check_hazard(&mut unknown, &mission.id, id);
        }
        for choice in &mission.route_choices {
            let where_ = format!("{}/{}", mission.id, choice.id);
            for id in &choice.enemy_add {
                check_enemy(&mut unknown, &where_, id);
            }
            for id in &choice.hazard_add {
                check_hazard(&mut unknown, &where_, id);
            }
        }
    }

    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!("unknown content ids -> {}", unknown.join(", ")))
    }
}

/// Verify the mission unlock graph is sound: every prerequisite/branch id
/// refers to a real mission, and every mission can eventually be unlocked from
/// a fresh campaign. A typo'd prerequisite otherwise silently strands a mission
/// as permanently locked. (Carriage-level gates are always satisfiable via
/// upgrades, so only completion prerequisites constrain reachability.)
pub fn validate_mission_reachability(missions: &[&MissionDef]) -> Result<(), String> {
    use std::collections::HashSet;

    let ids: HashSet<&str> = missions.iter().map(|mission| mission.id.as_str()).collect();
    let mut errors = Vec::new();

    for mission in missions {
        for id in mission
            .prerequisite_missions
            .iter()
            .chain(mission.unlock_any_missions.iter())
        {
            if !ids.contains(id.as_str()) {
                errors.push(format!(
                    "{}: references unknown mission '{}'",
                    mission.id, id
                ));
            }
        }
    }

    // Fixpoint: a mission unlocks once all its prerequisites are reachable and
    // (if it has a branch requirement) at least one branch is reachable.
    let mut reachable: HashSet<&str> = HashSet::new();
    loop {
        let mut changed = false;
        for mission in missions {
            if reachable.contains(mission.id.as_str()) {
                continue;
            }
            let prereqs_ok = mission
                .prerequisite_missions
                .iter()
                .all(|id| reachable.contains(id.as_str()));
            let branch_ok = mission.unlock_any_missions.is_empty()
                || mission
                    .unlock_any_missions
                    .iter()
                    .any(|id| reachable.contains(id.as_str()));
            if prereqs_ok && branch_ok {
                reachable.insert(mission.id.as_str());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for mission in missions {
        if !reachable.contains(mission.id.as_str()) {
            errors.push(format!(
                "{}: unreachable (its prerequisites can never all be met)",
                mission.id
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("mission graph invalid -> {}", errors.join("; ")))
    }
}
