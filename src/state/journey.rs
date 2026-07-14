//! Roguelite "expedition" mode: chained mission legs with carry-over carriage
//! damage, escalating difficulty, and a bank-or-push fail-state.
//!
//! An expedition reuses the campaign loadout (guards, chassis, equipment) but
//! runs missions back-to-back. Carriage damage carries between legs, difficulty
//! climbs each leg, and rewards bank as you go — cash out to keep them all, or
//! push on and risk losing half if a leg is lost. Expeditions are session-only
//! (not saved) and never touch campaign mission records.

use super::{GameSession, MissionReport, MissionRun, Screen};
use crate::data::{GameData, MissionDef, RelicDef};

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
    /// Ids of relics collected this run; folded into every leg's mission stats
    /// (see `GameSession::begin_journey_leg`). Session-only — never persisted.
    pub relics: Vec<String>,
    /// FTL-style branch: 2–3 bespoke next-leg options to choose between at the
    /// hub. `Some` from leg 2 onward (leg 1 auto-starts).
    pub pending_legs: Option<Vec<LegOption>>,
    /// The bespoke composition of the leg currently in progress (base route +
    /// modifier). `None` on the auto-started first leg.
    pub current_leg: Option<LegOption>,
    /// A between-legs vignette (run-event id) awaiting a decision at the hub.
    /// `Some` blocks the next-leg branch until resolved.
    pub pending_event: Option<String>,
    /// Flavor outcome of the most recently resolved run event (hub readout).
    pub last_event_result: Option<String>,
    /// True once the final leg has been cleared: the run is won and awaiting the
    /// victory summary. A won run still `alive`, so banking pays out in full.
    pub won: bool,
}

/// One branch in the expedition's next-leg choice: a base campaign route paired
/// with a bespoke [`crate::data::LegModifierDef`] twist. Resolved against the
/// data registries for display and application.
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

/// One of the three rewards offered after clearing an expedition leg. A relic is
/// a run-defining build pick; the others are trades between raw gold and carriage
/// upkeep. Which is best depends on how battered the convoy is — not a flat payout.
#[derive(Debug, Clone)]
pub enum LegReward {
    /// Pure gold, generous — the greedy pick with no upkeep.
    Bounty(i64),
    /// Modest gold plus a partial patch-up.
    Provisions { gold: i64, heal: f32 },
    /// A full carriage repair plus a little gold — best when badly damaged.
    Repair { gold: i64 },
    /// A run-scoped relic (id) that reshapes how the rest of the run plays.
    Relic(String),
}

impl LegReward {
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
            LegReward::Relic(id) => {
                journey.relics.push(id);
                journey.last_reward = 0;
            }
        }
    }

    /// Display title. Relics need the data registry to resolve their name.
    pub fn title(&self, data: &GameData) -> String {
        match self {
            LegReward::Bounty(_) => "Bounty Purse".to_owned(),
            LegReward::Provisions { .. } => "War Provisions".to_owned(),
            LegReward::Repair { .. } => "Field Repairs".to_owned(),
            LegReward::Relic(id) => data
                .relics
                .get(id)
                .map(|relic| format!("Relic — {}", relic.name))
                .unwrap_or_else(|| "Relic".to_owned()),
        }
    }

    pub fn detail(&self, data: &GameData) -> String {
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
            LegReward::Relic(id) => data
                .relics
                .get(id)
                .map(|relic| relic.description.clone())
                .unwrap_or_else(|| "A mysterious boon.".to_owned()),
        }
    }
}

impl Journey {
    /// Enemy/hazard difficulty multiplier for the current leg.
    pub fn difficulty_scale(&self) -> f32 {
        1.0 + (self.leg.saturating_sub(1)) as f32 * 0.12
    }

    /// Number of legs in a full expedition. Clearing the final leg wins the run
    /// (a defined arc + win condition, not an endless "leave or die" treadmill).
    pub const EXPEDITION_LENGTH: u32 = 8;

    /// Whether `leg` is the final leg of the expedition.
    pub fn is_final_leg(leg: u32) -> bool {
        leg >= Self::EXPEDITION_LENGTH
    }

    /// The victory bonus banked for completing the whole expedition.
    pub fn completion_bonus() -> i64 {
        Journey::leg_reward(Self::EXPEDITION_LENGTH) * 2
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

    /// Product of reward multipliers from every collected relic (1.0 with none).
    fn reward_mult(&self, data: &GameData) -> f32 {
        self.relics
            .iter()
            .filter_map(|id| data.relics.get(id))
            .map(|relic| relic.reward_mult)
            .product::<f32>()
            .max(0.1)
    }

    /// A relic not yet collected this run, picked deterministically by leg so the
    /// offer rotates. `None` once every relic has been collected.
    fn next_relic_offer(&self, data: &GameData) -> Option<String> {
        let available: Vec<&RelicDef> = data
            .relics_ordered()
            .into_iter()
            .filter(|relic| !self.relics.iter().any(|owned| owned == &relic.id))
            .collect();
        if available.is_empty() {
            return None;
        }
        let idx = (self.leg.saturating_sub(1) as usize) % available.len();
        Some(available[idx].id.clone())
    }

    /// The three reward choices offered after clearing the current leg. When an
    /// un-collected relic is available it takes the first slot (the build pick);
    /// otherwise that slot is a pure-gold Bounty. Relic reward multipliers are
    /// baked into the gold amounts shown.
    /// Multiplier the current leg's modifier applies to its banked reward.
    fn leg_reward_mult(&self, data: &GameData) -> f32 {
        self.current_leg
            .as_ref()
            .and_then(|opt| data.leg_modifiers.get(&opt.modifier_id))
            .map(|m| m.reward_mult)
            .unwrap_or(1.0)
    }

    /// 2–3 bespoke options for the next leg: rotating base routes each paired
    /// with a distinct modifier, chosen deterministically by leg so the branch
    /// is stable within a run but varies down the expedition.
    pub fn generate_leg_options(&self, data: &GameData) -> Vec<LegOption> {
        let missions = data.missions_ordered();
        let modifiers = data.leg_modifiers_ordered();
        if missions.is_empty() || modifiers.is_empty() {
            return Vec::new();
        }
        let count = 3.min(modifiers.len()).min(missions.len().max(1));
        (0..count)
            .map(|i| {
                let m_idx = (self.leg as usize + i) % missions.len();
                let mod_idx = (self.leg as usize + i * 2) % modifiers.len();
                LegOption {
                    mission_id: missions[m_idx].id.clone(),
                    modifier_id: modifiers[mod_idx].id.clone(),
                }
            })
            .collect()
    }

    /// The between-legs vignette to present before the next-leg branch, rotating
    /// through the pool by leg. `None` if no events are defined.
    pub fn next_run_event(&self, data: &GameData) -> Option<String> {
        let events = data.run_events_ordered();
        if events.is_empty() {
            return None;
        }
        Some(events[(self.leg as usize) % events.len()].id.clone())
    }

    pub fn leg_reward_choices(&self, data: &GameData) -> [LegReward; 3] {
        let base = Journey::leg_reward(self.leg);
        let mult = self.reward_mult(data) * self.leg_reward_mult(data);
        let gold = |raw: i64| ((raw as f32) * mult).round() as i64;
        let first = match self.next_relic_offer(data) {
            Some(id) => LegReward::Relic(id),
            None => LegReward::Bounty(gold(base + base / 2)),
        };
        [
            first,
            LegReward::Provisions {
                gold: gold(base),
                heal: 0.25,
            },
            LegReward::Repair {
                gold: gold(base / 3),
            },
        ]
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
            relics: Vec::new(),
            pending_legs: None,
            current_leg: None,
            pending_event: None,
            last_event_result: None,
            won: false,
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
        // A bespoke leg names its own base route; the auto-started first leg
        // (no `current_leg`) falls back to the modulo campaign cycle.
        let mission = match &journey.current_leg {
            Some(option) => data.missions.get(&option.mission_id),
            None => self.journey_mission(data),
        };
        let Some(mission) = mission else {
            return false;
        };
        let mut run = MissionRun::new(mission, &self.campaign);
        run.scale_for_journey(journey.difficulty_scale(), journey.carriage_health_ratio);
        if let Some(option) = &journey.current_leg {
            if let Some(modifier) = data.leg_modifiers.get(&option.modifier_id) {
                run.apply_leg_modifier(modifier);
            }
        }
        for id in &journey.relics {
            if let Some(relic) = data.relics.get(id) {
                run.apply_relic(relic);
            }
        }
        self.mission = Some(run);
        self.result = None;
        self.screen = Screen::Playing;
        true
    }

    /// Applies a completed leg's report to the active expedition. Success banks
    /// the reward and advances; failure ends the run with a half payout.
    pub(super) fn resolve_journey_leg(&mut self, report: &MissionReport, data: &GameData) {
        if self.journey.is_none() {
            return;
        }
        if report.success {
            // Offer a choice of rewards for the leg just cleared; advancing to
            // the next leg is gated on picking one (`journey_choose_reward`).
            let choices = self.journey.as_ref().unwrap().leg_reward_choices(data);
            let journey = self.journey.as_mut().unwrap();
            journey.last_mission_name = report.mission_name.clone();
            journey.carriage_health_ratio = report.carriage_health_ratio.max(0.05);
            journey.pending_rewards = Some(choices);
        } else {
            let journey = self.journey.as_mut().unwrap();
            journey.last_mission_name = report.mission_name.clone();
            journey.alive = false;
            journey.payout = journey.banked_gold / 2;
            let payout = journey.payout;
            self.campaign.gold += payout;
        }
        self.mission = None;
        self.screen = Screen::Journey;
    }

    /// Applies the chosen post-leg reward, advances the leg counter, and offers
    /// the bespoke branch of next-leg options. No-op if no rewards are pending or
    /// the index is out of range.
    pub fn journey_choose_reward(&mut self, index: usize, data: &GameData) -> bool {
        let Some(journey) = self.journey.as_mut() else {
            return false;
        };
        let Some(reward) = journey
            .pending_rewards
            .as_ref()
            .and_then(|rewards| rewards.get(index))
            .cloned()
        else {
            return false;
        };
        reward.apply(journey);
        journey.pending_rewards = None;
        if Journey::is_final_leg(journey.leg) {
            // The final leg's reward was just taken — the expedition is won.
            let bonus = Journey::completion_bonus();
            journey.banked_gold += bonus;
            journey.payout = bonus;
            journey.won = true;
        } else {
            journey.leg += 1;
            let options = journey.generate_leg_options(data);
            journey.pending_legs = Some(options);
            // A between-legs vignette precedes the branch (if any exist).
            journey.pending_event = journey.next_run_event(data);
        }
        true
    }

    /// Applies the chosen option of the pending run event and clears it. No-op if
    /// no event is pending or the indices don't resolve.
    pub fn journey_resolve_event(&mut self, index: usize, data: &GameData) -> bool {
        let Some(event_id) = self.journey.as_ref().and_then(|j| j.pending_event.clone()) else {
            return false;
        };
        let Some(event) = data.run_events.get(&event_id) else {
            return false;
        };
        let Some(option) = event.options.get(index) else {
            return false;
        };
        let Some(journey) = self.journey.as_mut() else {
            return false;
        };
        journey.banked_gold = (journey.banked_gold + option.gold).max(0);
        if option.health.abs() > f32::EPSILON {
            journey.carriage_health_ratio =
                (journey.carriage_health_ratio + option.health).clamp(0.05, 1.0);
        }
        if !option.relic.is_empty()
            && data.relics.contains(&option.relic)
            && !journey.relics.iter().any(|owned| owned == &option.relic)
        {
            journey.relics.push(option.relic.clone());
        }
        journey.last_event_result = Some(option.result.clone());
        journey.pending_event = None;
        true
    }

    /// Begins the chosen branch of the pending leg options. No-op while a reward
    /// is still pending or the index is out of range.
    pub fn journey_begin_leg(&mut self, index: usize, data: &GameData) -> bool {
        let ready = self.journey.as_ref().is_some_and(|journey| {
            journey.alive && journey.pending_rewards.is_none() && journey.pending_event.is_none()
        });
        if !ready {
            return false;
        }
        if let Some(journey) = self.journey.as_mut() {
            if let Some(option) = journey
                .pending_legs
                .as_ref()
                .and_then(|legs| legs.get(index))
                .cloned()
            {
                journey.current_leg = Some(option);
                journey.pending_legs = None;
            }
        }
        self.begin_journey_leg(data)
    }

    /// Convenience wrapper that begins the first offered leg branch.
    pub fn journey_press_on(&mut self, data: &GameData) -> bool {
        self.journey_begin_leg(0, data)
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
