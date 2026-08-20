//! Persistent expedition history and aggregate statistics.

use serde::{Deserialize, Serialize};

/// The most recent expeditions kept in the run-history log.
const RUN_HISTORY_CAP: usize = 8;

/// Persistent expedition statistics and recent-run history, saved on the
/// campaign. Powers the Records screen.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpeditionRecords {
    /// Expeditions ever started.
    #[serde(default)]
    pub runs_started: u32,
    /// Expeditions run to full completion (all legs cleared).
    #[serde(default)]
    pub wins: u32,
    /// Most legs cleared in a single run.
    #[serde(default)]
    pub best_legs: u32,
    /// Most gold banked from a single run.
    #[serde(default)]
    pub best_banked: i64,
    /// Legs cleared across all runs.
    #[serde(default)]
    pub total_legs: u32,
    /// Most recent runs, newest first, capped at `RUN_HISTORY_CAP`.
    #[serde(default)]
    pub history: Vec<ExpeditionRunSummary>,
}

/// A single completed expedition's outcome, for the history log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpeditionRunSummary {
    pub seed_code: String,
    pub seeded: bool,
    pub legs_cleared: u32,
    pub banked: i64,
    pub won: bool,
}

impl ExpeditionRecords {
    /// Folds a finished run into the records: bests, totals, and history.
    pub(super) fn record(&mut self, summary: ExpeditionRunSummary) {
        self.best_legs = self.best_legs.max(summary.legs_cleared);
        self.best_banked = self.best_banked.max(summary.banked);
        self.total_legs += summary.legs_cleared;
        if summary.won {
            self.wins += 1;
        }
        self.history.insert(0, summary);
        self.history.truncate(RUN_HISTORY_CAP);
    }
}
