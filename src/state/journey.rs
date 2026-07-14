//! Roguelite "expedition" mode: chained mission legs with carry-over carriage
//! damage, escalating difficulty, and a bank-or-push fail-state.
//!
//! An expedition reuses the campaign loadout (guards, chassis, equipment) but
//! runs missions back-to-back. Carriage damage carries between legs, difficulty
//! climbs each leg, and rewards bank as you go — cash out to keep them all, or
//! push on and risk losing half if a leg is lost. Expeditions are session-only
//! (not saved) and never touch campaign mission records.

use super::{GameSession, MissionReport, MissionRun, Screen};
use crate::data::{GameData, MissionDef};

#[derive(Debug, Clone)]
pub struct Journey {
    /// Current leg number, 1-based.
    pub leg: u32,
    /// Rewards earned so far this run, not yet paid to the campaign.
    pub banked_gold: i64,
    /// Carriage health carried into the next leg, as a fraction of max.
    pub carriage_health_ratio: f32,
    /// False once a leg has been lost; the run is over and awaiting summary.
    pub alive: bool,
    /// Reward from the most recently completed leg (for the hub display).
    pub last_reward: i64,
    /// Name of the most recent leg's route.
    pub last_mission_name: String,
    /// Gold actually paid out when the run ended (failure summary).
    pub payout: i64,
    /// Reward options offered after clearing a leg, awaiting the player's pick.
    /// `Some` blocks pressing on until one is chosen.
    pub pending_rewards: Option<[LegReward; 3]>,
}

/// One of the three rewards offered after clearing an expedition leg. Each is a
/// different trade between raw gold and carriage upkeep, so the pick depends on
/// how battered the convoy is — not a flat payout.
#[derive(Debug, Clone, Copy)]
pub enum LegReward {
    /// Pure gold, generous — the greedy pick with no upkeep.
    Bounty(i64),
    /// Modest gold plus a partial patch-up.
    Provisions { gold: i64, heal: f32 },
    /// A full carriage repair plus a little gold — best when badly damaged.
    Repair { gold: i64 },
}

impl LegReward {
    /// The three choices offered for clearing `leg`, scaled off its base reward.
    pub fn choices(leg: u32) -> [LegReward; 3] {
        let base = Journey::leg_reward(leg);
        [
            LegReward::Bounty(base + base / 2),
            LegReward::Provisions {
                gold: base,
                heal: 0.25,
            },
            LegReward::Repair { gold: base / 3 },
        ]
    }

    /// Applies this reward to the run and records it as the last leg reward.
    fn apply(self, journey: &mut Journey) {
        match self {
            LegReward::Bounty(gold) => {
                journey.banked_gold += gold;
                journey.last_reward = gold;
            }
            LegReward::Provisions { gold, heal } => {
                journey.banked_gold += gold;
                journey.carriage_health_ratio = (journey.carriage_health_ratio + heal).min(1.0);
                journey.last_reward = gold;
            }
            LegReward::Repair { gold } => {
                journey.banked_gold += gold;
                journey.carriage_health_ratio = 1.0;
                journey.last_reward = gold;
            }
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            LegReward::Bounty(_) => "Bounty Purse",
            LegReward::Provisions { .. } => "War Provisions",
            LegReward::Repair { .. } => "Field Repairs",
        }
    }

    pub fn detail(self) -> String {
        match self {
            LegReward::Bounty(gold) => format!("+{} gold banked", gold),
            LegReward::Provisions { gold, heal } => {
                format!(
                    "+{} gold, +{}% carriage health",
                    gold,
                    (heal * 100.0) as i32
                )
            }
            LegReward::Repair { gold } => format!("Full repair, +{} gold", gold),
        }
    }
}

impl Journey {
    /// Enemy/hazard difficulty multiplier for the current leg.
    pub fn difficulty_scale(&self) -> f32 {
        1.0 + (self.leg.saturating_sub(1)) as f32 * 0.12
    }

    /// Gold banked for completing a given leg (escalates with depth).
    pub fn leg_reward(leg: u32) -> i64 {
        30 + leg as i64 * 22
    }

    /// Cost to fully repair the carriage from its current carry-over health,
    /// paid out of banked gold.
    pub fn repair_cost(&self) -> i64 {
        (((1.0 - self.carriage_health_ratio) * 90.0).round() as i64).max(1)
    }

    pub fn can_repair(&self) -> bool {
        self.alive && self.carriage_health_ratio < 0.995 && self.banked_gold >= self.repair_cost()
    }
}

impl GameSession {
    pub fn start_journey(&mut self, data: &GameData) -> bool {
        self.journey = Some(Journey {
            leg: 1,
            banked_gold: 0,
            carriage_health_ratio: 1.0,
            alive: true,
            last_reward: 0,
            last_mission_name: String::new(),
            payout: 0,
            pending_rewards: None,
        });
        self.begin_journey_leg(data)
    }

    fn journey_mission<'a>(&self, data: &'a GameData) -> Option<&'a MissionDef> {
        let leg = self.journey.as_ref()?.leg;
        let pool = data.missions_ordered();
        if pool.is_empty() {
            return None;
        }
        Some(pool[((leg - 1) as usize) % pool.len()])
    }

    fn begin_journey_leg(&mut self, data: &GameData) -> bool {
        let Some(journey) = self.journey.clone() else {
            return false;
        };
        let Some(mission) = self.journey_mission(data) else {
            return false;
        };
        let mut run = MissionRun::new(mission, &self.campaign);
        run.scale_for_journey(journey.difficulty_scale(), journey.carriage_health_ratio);
        self.mission = Some(run);
        self.result = None;
        self.screen = Screen::Playing;
        true
    }

    /// Applies a completed leg's report to the active expedition. Success banks
    /// the reward and advances; failure ends the run with a half payout.
    pub(super) fn resolve_journey_leg(&mut self, report: &MissionReport) {
        let Some(journey) = self.journey.as_mut() else {
            return;
        };
        journey.last_mission_name = report.mission_name.clone();
        if report.success {
            journey.carriage_health_ratio = report.carriage_health_ratio.max(0.05);
            // Offer a choice of rewards for the leg just cleared; advancing to
            // the next leg is gated on picking one (`journey_choose_reward`).
            journey.pending_rewards = Some(LegReward::choices(journey.leg));
        } else {
            journey.alive = false;
            journey.payout = journey.banked_gold / 2;
            let payout = journey.payout;
            self.campaign.gold += payout;
        }
        self.mission = None;
        self.screen = Screen::Journey;
    }

    /// Applies the chosen post-leg reward and advances to the next leg. No-op if
    /// no rewards are pending or the index is out of range.
    pub fn journey_choose_reward(&mut self, index: usize) -> bool {
        let Some(journey) = self.journey.as_mut() else {
            return false;
        };
        let Some(reward) = journey
            .pending_rewards
            .and_then(|rewards| rewards.get(index).copied())
        else {
            return false;
        };
        reward.apply(journey);
        journey.pending_rewards = None;
        journey.leg += 1;
        true
    }

    pub fn journey_press_on(&mut self, data: &GameData) -> bool {
        // A pending reward must be resolved before the next leg can begin.
        if self
            .journey
            .as_ref()
            .is_some_and(|journey| journey.alive && journey.pending_rewards.is_none())
        {
            self.begin_journey_leg(data)
        } else {
            false
        }
    }

    pub fn journey_repair(&mut self) -> bool {
        let Some(journey) = self.journey.as_mut() else {
            return false;
        };
        if !journey.can_repair() {
            return false;
        }
        journey.banked_gold -= journey.repair_cost();
        journey.carriage_health_ratio = 1.0;
        true
    }

    /// Ends the expedition. A surviving run banks its full earnings; a failed
    /// run's payout was already applied when it ended.
    pub fn journey_bank_and_return(&mut self) {
        if let Some(journey) = self.journey.take() {
            if journey.alive {
                self.campaign.gold += journey.banked_gold;
            }
        }
        self.mission = None;
        self.screen = Screen::MissionMap;
    }
}
