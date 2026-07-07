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
            let reward = Journey::leg_reward(journey.leg);
            journey.banked_gold += reward;
            journey.last_reward = reward;
            journey.carriage_health_ratio = report.carriage_health_ratio.max(0.05);
            journey.leg += 1;
        } else {
            journey.alive = false;
            journey.payout = journey.banked_gold / 2;
            let payout = journey.payout;
            self.campaign.gold += payout;
        }
        self.mission = None;
        self.screen = Screen::Journey;
    }

    pub fn journey_press_on(&mut self, data: &GameData) -> bool {
        if self.journey.as_ref().is_some_and(|journey| journey.alive) {
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
