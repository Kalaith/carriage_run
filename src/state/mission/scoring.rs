//! Mission result scoring.

use super::*;
use crate::data::MissionDef;

impl MissionRun {
    pub(super) fn make_report(
        &self,
        mission: &MissionDef,
        success: bool,
        reason: &str,
    ) -> MissionReport {
        let health_ratio = (self.carriage.health / self.carriage.max_health).clamp(0.0, 1.0);
        let cargo_ratio = (self.carriage.cargo / self.carriage.max_cargo).clamp(0.0, 1.0);
        let special_ratio = self.special_ratio();
        let expected_time = self.distance / 17.5;
        let time_ratio = (1.0 - self.elapsed / (expected_time * 1.35)).clamp(0.0, 1.0);
        let base_score = health_ratio * 320.0
            + cargo_ratio * 320.0
            + time_ratio * 220.0
            + special_ratio.unwrap_or(0.0) * 140.0
            + self.enemies_defeated as f32 * 22.0
            - self.guard_damage_taken * 0.75;
        let score = if success {
            base_score.max(80.0).round() as i64
        } else {
            (base_score * 0.25).max(0.0).round() as i64
        };
        let stars = if !success {
            0
        } else if health_ratio > 0.68
            && cargo_ratio > 0.84
            && time_ratio > 0.25
            && special_ratio.unwrap_or(1.0) > 0.72
        {
            3
        } else if health_ratio > 0.38 && cargo_ratio > 0.62 && special_ratio.unwrap_or(1.0) > 0.42 {
            2
        } else {
            1
        };
        let reward = if success {
            self.base_reward
                + stars as i64 * 32
                + (cargo_ratio * 42.0).round() as i64
                + special_ratio
                    .map(|ratio| (ratio * 28.0).round() as i64)
                    .unwrap_or(0)
                + self.enemies_defeated as i64 * 4
        } else {
            (self.base_reward as f32 * 0.12).round() as i64 + self.enemies_defeated as i64 * 2
        };
        // A failed run costs gold: emergency repairs scale with carriage
        // damage, and spoiled/looted cargo scales with cargo lost. The worse
        // the run went, the more it stings.
        let gold_penalty = if success {
            0
        } else {
            let repairs = (1.0 - health_ratio) * 30.0;
            let lost_cargo = (1.0 - cargo_ratio) * 20.0;
            (repairs + lost_cargo).round() as i64
        };

        MissionReport {
            mission_id: mission.id.clone(),
            mission_name: mission.name.clone(),
            route_name: self.route_name.clone(),
            success,
            reason: reason.to_owned(),
            stars,
            score,
            reward,
            gold_penalty,
            elapsed: self.elapsed,
            time_limit: self.time_limit,
            carriage_health_ratio: health_ratio,
            cargo_ratio,
            special_label: self.mission_kind.label().map(ToOwned::to_owned),
            special_ratio,
            enemies_defeated: self.enemies_defeated,
            injured_guard_ids: injured_guard_ids(&self.guards),
        }
    }
}

fn injured_guard_ids(guards: &[Guard]) -> Vec<String> {
    let mut ids = Vec::new();
    for guard in guards {
        if guard.health <= 0.0 && !ids.iter().any(|id: &String| id == guard.kind.id()) {
            ids.push(guard.kind.id().to_owned());
        }
    }
    ids
}
