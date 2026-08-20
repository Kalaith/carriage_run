//! Session-scoped expedition branches and reward choices.

use super::Journey;
use crate::data::GameData;

/// One branch in the expedition's next-leg choice: a base campaign route paired
/// with a bespoke [`crate::data::LegModifierDef`] twist.
#[derive(Debug, Clone)]
pub struct LegOption {
    pub mission_id: String,
    pub modifier_id: String,
}

impl LegOption {
    /// The modifier's name, e.g. "Raider Ambush".
    pub fn title(&self, data: &GameData) -> String {
        data.leg_modifiers
            .get(&self.modifier_id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "Onward".to_owned())
    }
}

/// One of the three rewards offered after clearing an expedition leg. A relic
/// is a run-defining build pick; the others trade raw gold and upkeep.
#[derive(Debug, Clone)]
pub enum LegReward {
    Bounty(i64),
    Provisions { gold: i64, heal: f32 },
    Repair { gold: i64 },
    Relic(String),
}

impl LegReward {
    /// Applies this reward to the run and records it as the last leg reward.
    pub(super) fn apply(self, journey: &mut Journey) {
        match self {
            Self::Bounty(gold) => {
                journey.banked_gold += gold;
                journey.last_reward = gold;
            }
            Self::Provisions { gold, heal } => {
                journey.banked_gold += gold;
                journey.carriage_health_ratio = (journey.carriage_health_ratio + heal).min(1.0);
                journey.last_reward = gold;
            }
            Self::Repair { gold } => {
                journey.banked_gold += gold;
                journey.carriage_health_ratio = 1.0;
                journey.last_reward = gold;
            }
            Self::Relic(id) => {
                journey.relics.push(id);
                journey.last_reward = 0;
            }
        }
    }

    pub fn title(&self, data: &GameData) -> String {
        match self {
            Self::Bounty(_) => "Bounty Purse".to_owned(),
            Self::Provisions { .. } => "War Provisions".to_owned(),
            Self::Repair { .. } => "Field Repairs".to_owned(),
            Self::Relic(id) => data
                .relics
                .get(id)
                .map(|relic| format!("Relic — {}", relic.name))
                .unwrap_or_else(|| "Relic".to_owned()),
        }
    }

    pub fn detail(&self, data: &GameData) -> String {
        match self {
            Self::Bounty(gold) => format!("+{} gold banked", gold),
            Self::Provisions { gold, heal } => {
                format!(
                    "+{} gold, +{}% carriage health",
                    gold,
                    (heal * 100.0) as i32
                )
            }
            Self::Repair { gold } => format!("Full repair, +{} gold", gold),
            Self::Relic(id) => data
                .relics
                .get(id)
                .map(|relic| relic.description.clone())
                .unwrap_or_else(|| "A mysterious boon.".to_owned()),
        }
    }
}
