//! Campaign progression helpers for mission paths and route choices.

use super::CampaignState;
use crate::data::{MissionDef, RouteChoiceDef};

impl CampaignState {
    pub fn is_mission_unlocked(&self, mission: &MissionDef) -> bool {
        self.carriage_level >= mission.unlock_level
            && mission
                .prerequisite_missions
                .iter()
                .all(|id| self.is_mission_completed(id))
            && (mission.unlock_any_missions.is_empty()
                || mission
                    .unlock_any_missions
                    .iter()
                    .any(|id| self.is_mission_completed(id)))
    }

    pub fn mission_unlock_label(&self, mission: &MissionDef) -> String {
        if self.carriage_level < mission.unlock_level {
            return format!("Carriage Level {}", mission.unlock_level);
        }

        if let Some(id) = mission
            .prerequisite_missions
            .iter()
            .find(|id| !self.is_mission_completed(id))
        {
            return format!("Complete {}", format_mission_id(id));
        }

        if !mission.unlock_any_missions.is_empty()
            && !mission
                .unlock_any_missions
                .iter()
                .any(|id| self.is_mission_completed(id))
        {
            let first = mission
                .unlock_any_missions
                .first()
                .map(|id| format_mission_id(id))
                .unwrap_or_else(|| "a route".to_owned());
            return format!("Complete {} branch", first);
        }

        "Locked".to_owned()
    }

    pub fn is_mission_completed(&self, id: &str) -> bool {
        self.records
            .get(id)
            .is_some_and(|record| record.completions > 0)
    }

    pub fn selected_route_choice<'a>(&self, mission: &'a MissionDef) -> Option<&'a RouteChoiceDef> {
        self.selected_route_choices
            .get(&mission.id)
            .and_then(|id| mission.route_choices.iter().find(|choice| choice.id == *id))
            .or_else(|| mission.route_choices.first())
    }

    pub fn selected_route_choice_id<'a>(&self, mission: &'a MissionDef) -> Option<&'a str> {
        self.selected_route_choice(mission)
            .map(|choice| choice.id.as_str())
    }

    pub fn select_route_choice(&mut self, mission: &MissionDef, route_id: &str) -> bool {
        if !mission
            .route_choices
            .iter()
            .any(|choice| choice.id == route_id)
        {
            return false;
        }

        self.selected_route_choices
            .insert(mission.id.clone(), route_id.to_owned());
        true
    }
}

fn format_mission_id(id: &str) -> String {
    id.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
