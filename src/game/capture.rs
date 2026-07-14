//! Scene seeding for the headless screenshot harness (see `run_capture`).

use super::Game;
use crate::state::{
    CodexTab, ConfirmPrompt, ExpeditionRunSummary, Journey, MissionRecord, MissionReport,
    MissionRun, Screen,
};

impl Game {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "map" => {
                // Seed a few cleared routes so the header progress reads
                // non-zero and more of the map is unlocked.
                for (id, stars) in [
                    ("muddy_road", 3),
                    ("bandit_bend", 2),
                    ("courier_deadline", 3),
                ] {
                    self.session.campaign.records.insert(
                        id.to_owned(),
                        MissionRecord {
                            best_stars: stars,
                            best_score: 540,
                            best_reward: 150,
                            completions: 1,
                        },
                    );
                }
                self.session.open_map();
            }
            "loadout" => self.session.open_loadout(),
            "upgrades" => self.session.open_upgrades(),
            "carriages" => self.session.open_carriages(),
            "guards" => {
                // Seed an injured guard so the infirmary UI is visible.
                self.session
                    .campaign
                    .guard_recovery
                    .insert("swordsman".to_owned(), 2);
                self.session.open_guards();
            }
            "title" => self.session.return_title(),
            "shop" => {
                // Give some gold so the provisions Buy button is enabled.
                self.session.campaign.gold = 300;
                self.session.campaign.reinforced_kits = 2;
                self.session.open_shop();
            }
            "settings" => self.session.open_settings(),
            "codex" => self.session.open_codex(),
            "codexguards" => {
                self.session.open_codex();
                self.session.set_codex_tab(CodexTab::Guards);
            }
            "codexhazards" => {
                self.session.open_codex();
                self.session.set_codex_tab(CodexTab::Hazards);
            }
            "results" => {
                // A completed mission with a special meter (most stat rows) so
                // the results layout is exercised at its fullest.
                self.session.result = Some(MissionReport {
                    mission_id: "medicine_run".to_owned(),
                    mission_name: "Medicine Run".to_owned(),
                    route_name: "Dry Ridge".to_owned(),
                    success: true,
                    reason: "Delivered before the medicine spoiled".to_owned(),
                    stars: 2,
                    score: 742,
                    reward: 214,
                    gold_penalty: 0,
                    elapsed: 63.0,
                    time_limit: Some(86.0),
                    carriage_health_ratio: 0.71,
                    cargo_ratio: 0.88,
                    special_label: Some("Potency".to_owned()),
                    special_ratio: Some(0.79),
                    enemies_defeated: 11,
                    injured_guard_ids: Vec::new(),
                    bonus_met: Some(true),
                });
                self.session.screen = Screen::Results;
            }
            "confirm" => {
                // Title screen with the New Campaign overwrite prompt staged.
                self.session.return_title();
                self.session.pending_confirm = Some(ConfirmPrompt::NewCampaign);
            }
            "journey" => {
                // Seed a mid-run expedition hub with a bespoke branch to choose.
                let journey = Journey {
                    last_reward: 66,
                    relics: vec!["ghost_wheels".to_owned(), "merchants_ledger".to_owned()],
                    ..mid_run_journey()
                };
                let legs = journey.generate_leg_options(&self.data);
                self.session.journey = Some(Journey {
                    pending_legs: Some(legs),
                    ..journey
                });
                self.session.screen = Screen::Journey;
            }
            "journey_reward" => {
                // Seed the post-leg reward-choice screen (with a relic on offer).
                let journey = mid_run_journey();
                let choices = journey.leg_reward_choices(&self.data);
                self.session.journey = Some(Journey {
                    pending_rewards: Some(choices),
                    ..journey
                });
                self.session.screen = Screen::Journey;
            }
            "journey_event" => {
                // Seed the between-legs vignette (run-event) decision screen.
                let journey = mid_run_journey();
                let event = journey.next_run_event(&self.data);
                self.session.journey = Some(Journey {
                    pending_event: event,
                    ..journey
                });
                self.session.screen = Screen::Journey;
            }
            "outfitter" => {
                // Seed the pre-expedition Outfitter with tokens and one unlock.
                self.session.campaign.expedition_tokens = 14;
                self.session
                    .campaign
                    .expedition_unlocks
                    .push("greased_axles".to_owned());
                self.session.open_outfitter();
            }
            "records" => {
                // Seed the Records screen with lifetime stats and run history.
                let records = &mut self.session.campaign.expedition_records;
                records.runs_started = 12;
                records.wins = 3;
                records.best_legs = 8;
                records.best_banked = 742;
                records.total_legs = 47;
                records.history = vec![
                    ExpeditionRunSummary {
                        seed_code: "1A2B3C4D".to_owned(),
                        seeded: true,
                        legs_cleared: 8,
                        banked: 742,
                        won: true,
                    },
                    ExpeditionRunSummary {
                        seed_code: "00000000".to_owned(),
                        seeded: false,
                        legs_cleared: 5,
                        banked: 318,
                        won: false,
                    },
                    ExpeditionRunSummary {
                        seed_code: "00000000".to_owned(),
                        seeded: false,
                        legs_cleared: 3,
                        banked: 210,
                        won: false,
                    },
                ];
                self.session.open_records();
            }
            "journey_win" => {
                // Seed the expedition-victory summary screen.
                self.session.journey = Some(Journey {
                    leg: Journey::EXPEDITION_LENGTH,
                    banked_gold: 640,
                    carriage_health_ratio: 0.44,
                    last_mission_name: "Ashford Gate".to_owned(),
                    payout: Journey::completion_bonus(),
                    relics: vec!["ghost_wheels".to_owned(), "spiked_ram".to_owned()],
                    won: true,
                    legs_cleared: Journey::EXPEDITION_LENGTH,
                    seed: 0x1A2B3C4D,
                    seeded: true,
                    ..mid_run_journey()
                });
                self.session.screen = Screen::Journey;
            }
            "princess" => {
                // Drop straight into the princess "drive clean" mission so the
                // HUD smoothness multiplier is visible (bypasses unlock gates).
                if let Some(mission) = self.data.missions.get("princess_road") {
                    self.session.mission = Some(MissionRun::new(mission, &self.session.campaign));
                    self.session.screen = Screen::Playing;
                } else {
                    self.session.open_map();
                }
            }
            _ => {
                self.session.select_mission("muddy_road");
                if !self.session.start_selected_mission(&self.data) {
                    self.session.open_map();
                }
            }
        }
    }
}

/// A partway-through expedition: two legs cleared, carriage worn down, gold
/// banked. Scenes override the fields their screen actually shows.
fn mid_run_journey() -> Journey {
    Journey {
        leg: 3,
        banked_gold: 148,
        carriage_health_ratio: 0.52,
        alive: true,
        last_reward: 0,
        last_mission_name: "Bandit Bend".to_owned(),
        payout: 0,
        pending_rewards: None,
        relics: Vec::new(),
        pending_legs: None,
        current_leg: None,
        pending_event: None,
        last_event_result: None,
        won: false,
        legs_cleared: 2,
        seed: 0,
        seeded: false,
        stake_mult: 1.0,
    }
}
