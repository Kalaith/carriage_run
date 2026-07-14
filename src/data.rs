//! Embedded game data and asset manifests.

use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::{
    load_embedded_json, load_embedded_json_labeled, DataRegistry,
};
use serde::{Deserialize, Serialize};

const GAME_CONFIG_JSON: &str = include_str!("../assets/data/game_config.json");
const MISSIONS_JSON: &str = include_str!("../assets/data/missions.json");
const UPGRADES_JSON: &str = include_str!("../assets/data/upgrades.json");
const CARRIAGES_JSON: &str = include_str!("../assets/data/carriages.json");
const TEXTURE_MANIFEST_JSON: &str = include_str!("../assets/data/texture_manifest.json");
const RELICS_JSON: &str = include_str!("../assets/data/relics.json");
const LEG_MODIFIERS_JSON: &str = include_str!("../assets/data/leg_modifiers.json");

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_name: String,
    pub display_name: String,
    pub save_slot: String,
    pub version: String,
    pub starting_gold: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionDef {
    pub id: String,
    pub name: String,
    pub order: u32,
    pub mission_type: String,
    pub route: String,
    pub cargo: String,
    pub objective: String,
    pub bonus_objective: String,
    /// One- or two-sentence courier-log vignette shown on the loadout brief;
    /// connects the missions into a single journey (light flavor, not plot).
    #[serde(default)]
    pub intro_text: String,
    /// Machine-evaluable target behind `bonus_objective`, so the results screen
    /// can report met/missed instead of the objective being flavor only.
    #[serde(default)]
    pub bonus: Option<BonusCriteria>,
    /// One-line courier-log payoff shown on the results screen after a
    /// successful run, bookending `intro_text`.
    #[serde(default)]
    pub outro_text: String,
    pub unlock_level: u32,
    pub distance: f32,
    pub difficulty: f32,
    pub base_reward: i64,
    pub enemy_mix: Vec<String>,
    pub hazard_mix: Vec<String>,
    #[serde(default)]
    pub route_choices: Vec<RouteChoiceDef>,
    #[serde(default)]
    pub prerequisite_missions: Vec<String>,
    #[serde(default)]
    pub unlock_any_missions: Vec<String>,
    #[serde(default)]
    pub time_limit: Option<f32>,
}

/// The measurable quantity a bonus objective is graded on, evaluated at
/// mission end in `MissionRun::make_report`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BonusMetric {
    /// Cargo remaining, as a 0..1 ratio.
    Cargo,
    /// Carriage health, as a 0..1 ratio.
    Health,
    /// Mission-specific meter (security / potency / comfort / …), 0..1 ratio.
    Special,
    /// Count of threats defeated.
    Threats,
    /// Seconds still on the clock at arrival (timed missions only).
    TimeRemaining,
}

/// A bonus objective's pass condition: `metric` must be at least `threshold`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BonusCriteria {
    pub metric: BonusMetric,
    pub threshold: f32,
}

impl BonusCriteria {
    /// Evaluate against a run's end-state metrics. `Special`/`TimeRemaining`
    /// return `false` when their value is absent (no meter / untimed mission).
    pub fn is_met(
        &self,
        cargo_ratio: f32,
        health_ratio: f32,
        special_ratio: Option<f32>,
        enemies_defeated: u32,
        seconds_remaining: Option<f32>,
    ) -> bool {
        let value = match self.metric {
            BonusMetric::Cargo => Some(cargo_ratio),
            BonusMetric::Health => Some(health_ratio),
            BonusMetric::Special => special_ratio,
            BonusMetric::Threats => Some(enemies_defeated as f32),
            BonusMetric::TimeRemaining => seconds_remaining,
        };
        value.is_some_and(|value| value >= self.threshold)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteChoiceDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub distance_delta: f32,
    #[serde(default)]
    pub difficulty_delta: f32,
    #[serde(default)]
    pub reward_delta: i64,
    #[serde(default)]
    pub time_limit_delta: f32,
    #[serde(default)]
    pub enemy_add: Vec<String>,
    #[serde(default)]
    pub hazard_add: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: i64,
    pub max_level: u32,
}

/// A purchasable carriage chassis. Determines guard/equipment slot count and
/// the carriage's speed and health multipliers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChassisDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: u32,
    pub slots: usize,
    pub speed_mult: f32,
    pub health_mult: f32,
    pub cost: i64,
}

/// A run-scoped expedition relic: a modifier collected during an expedition
/// that reshapes how that run plays (speed/armor/economy trades). Relics are
/// session-only and never touch the campaign. All effect fields are optional so
/// a relic tweaks only the axes it names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelicDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: u32,
    /// Multiplies carriage speed (1.0 = no change).
    #[serde(default = "one")]
    pub speed_mult: f32,
    /// Added to the carriage's damage-reduction fraction (can be negative).
    #[serde(default)]
    pub armor_add: f32,
    /// Added to the wheel bonus (faster cruise + hazard slow resistance).
    #[serde(default)]
    pub wheel_bonus_add: f32,
    /// Added contact damage per second to enemies hugging the carriage.
    #[serde(default)]
    pub hub_damage_add: f32,
    /// Multiplies gold from leg rewards (1.0 = no change).
    #[serde(default = "one")]
    pub reward_mult: f32,
}

/// A bespoke-expedition-leg archetype: a themed twist layered onto a base
/// campaign route when composing a procedural expedition leg (extra enemies /
/// hazards and difficulty/reward scaling). Drives the FTL-style branch choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegModifierDef {
    pub id: String,
    pub name: String,
    /// One-line flavor shown under the option in the branch picker.
    pub descriptor: String,
    pub order: u32,
    #[serde(default)]
    pub enemy_add: Vec<String>,
    #[serde(default)]
    pub hazard_add: Vec<String>,
    /// Multiplies the leg's mission difficulty (1.0 = no change).
    #[serde(default = "one")]
    pub difficulty_mult: f32,
    /// Multiplies the leg's banked reward (1.0 = no change).
    #[serde(default = "one")]
    pub reward_mult: f32,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub missions: DataRegistry<MissionDef>,
    pub upgrades: DataRegistry<UpgradeDef>,
    pub chassis: DataRegistry<ChassisDef>,
    pub relics: DataRegistry<RelicDef>,
    pub leg_modifiers: DataRegistry<LegModifierDef>,
    pub texture_manifest: Vec<TextureConfig>,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let config = load_embedded_json_labeled("game_config", GAME_CONFIG_JSON)?;
        let missions = DataRegistry::from_embedded_json(MISSIONS_JSON, "id")?;
        let upgrades = DataRegistry::from_embedded_json(UPGRADES_JSON, "id")?;
        let chassis = DataRegistry::from_embedded_json(CARRIAGES_JSON, "id")?;
        let relics = DataRegistry::from_embedded_json(RELICS_JSON, "id")?;
        let leg_modifiers = DataRegistry::from_embedded_json(LEG_MODIFIERS_JSON, "id")?;
        let texture_manifest = load_embedded_json(TEXTURE_MANIFEST_JSON)?;

        Ok(Self {
            config,
            missions,
            upgrades,
            chassis,
            relics,
            leg_modifiers,
            texture_manifest,
        })
    }

    pub fn relics_ordered(&self) -> Vec<&RelicDef> {
        let mut relics: Vec<_> = self.relics.iter().map(|(_, relic)| relic).collect();
        relics.sort_by_key(|relic| relic.order);
        relics
    }

    pub fn leg_modifiers_ordered(&self) -> Vec<&LegModifierDef> {
        let mut mods: Vec<_> = self.leg_modifiers.iter().map(|(_, m)| m).collect();
        mods.sort_by_key(|m| m.order);
        mods
    }

    pub fn first_mission_id(&self) -> Option<&str> {
        self.missions_ordered()
            .first()
            .map(|mission| mission.id.as_str())
    }

    pub fn missions_ordered(&self) -> Vec<&MissionDef> {
        let mut missions: Vec<_> = self.missions.iter().map(|(_, mission)| mission).collect();
        missions.sort_by_key(|mission| mission.order);
        missions
    }

    pub fn chassis_ordered(&self) -> Vec<&ChassisDef> {
        let mut chassis: Vec<_> = self.chassis.iter().map(|(_, chassis)| chassis).collect();
        chassis.sort_by_key(|chassis| chassis.order);
        chassis
    }

    /// The starter chassis every campaign begins with (lowest order).
    pub fn default_chassis_id(&self) -> String {
        self.chassis_ordered()
            .first()
            .map(|chassis| chassis.id.clone())
            .unwrap_or_else(|| "scout_cart".to_owned())
    }

    /// Best-fit chassis for a legacy save's carriage level, so migrated saves
    /// keep the slot count they had before chassis existed.
    pub fn chassis_for_level(&self, carriage_level: u32) -> String {
        let target_slots = if carriage_level >= 4 {
            4
        } else if carriage_level >= 2 {
            3
        } else {
            2
        };
        self.chassis_ordered()
            .into_iter()
            .find(|chassis| chassis.slots >= target_slots)
            .or_else(|| self.chassis_ordered().into_iter().last())
            .map(|chassis| chassis.id.clone())
            .unwrap_or_else(|| self.default_chassis_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_data_loads() {
        let data = GameData::load().unwrap();

        assert_eq!(data.config.game_name, "carriage_run");
        assert!(data.missions.contains("muddy_road"));
        assert!(data.missions.contains("siege_supply"));
        assert!(data.upgrades.contains("carriage_armor"));
        assert!(data.upgrades.contains("spiked_hubs"));
        assert!(data.upgrades.contains("warding_lantern"));
        assert_eq!(data.missions_ordered()[0].id, "muddy_road");
        assert!(data
            .missions_ordered()
            .iter()
            .all(|mission| !mission.intro_text.is_empty() && !mission.outro_text.is_empty()));
        assert_eq!(data.default_chassis_id(), "scout_cart");
        assert_eq!(data.chassis_ordered().len(), 3);
        assert_eq!(data.chassis_for_level(4), "heavy_wagon");
        assert_eq!(data.chassis_for_level(1), "scout_cart");
        assert!(!data
            .missions
            .get("bandit_bend")
            .unwrap()
            .route_choices
            .is_empty());
        assert!(data
            .texture_manifest
            .iter()
            .any(|texture| texture.key == "title_screen"));
    }

    #[test]
    fn every_mission_has_structured_bonus_criteria() {
        let data = GameData::load().unwrap();
        assert!(data
            .missions_ordered()
            .iter()
            .all(|mission| mission.bonus.is_some()));
    }

    #[test]
    fn bonus_criteria_evaluates_each_metric() {
        let cargo = BonusCriteria {
            metric: BonusMetric::Cargo,
            threshold: 0.85,
        };
        assert!(cargo.is_met(0.90, 0.0, None, 0, None));
        assert!(!cargo.is_met(0.80, 1.0, None, 99, None));

        let threats = BonusCriteria {
            metric: BonusMetric::Threats,
            threshold: 8.0,
        };
        assert!(threats.is_met(0.0, 0.0, None, 8, None));
        assert!(!threats.is_met(1.0, 1.0, None, 7, None));

        // Special/time metrics miss when their value is absent.
        let special = BonusCriteria {
            metric: BonusMetric::Special,
            threshold: 0.70,
        };
        assert!(special.is_met(0.0, 0.0, Some(0.71), 0, None));
        assert!(!special.is_met(1.0, 1.0, None, 0, None));

        let time = BonusCriteria {
            metric: BonusMetric::TimeRemaining,
            threshold: 12.0,
        };
        assert!(time.is_met(0.0, 0.0, None, 0, Some(13.0)));
        assert!(!time.is_met(1.0, 1.0, None, 0, Some(4.0)));
        assert!(!time.is_met(1.0, 1.0, None, 0, None));
    }

    #[test]
    fn mission_unlock_levels_are_non_decreasing_by_order() {
        let data = GameData::load().unwrap();
        for pair in data.missions_ordered().windows(2) {
            let (prev, next) = (pair[0], pair[1]);
            assert!(
                next.unlock_level >= prev.unlock_level,
                "unlock level regresses: '{}' (order {}) L{} follows '{}' (order {}) L{}",
                next.id,
                next.order,
                next.unlock_level,
                prev.id,
                prev.order,
                prev.unlock_level,
            );
            assert!(
                next.base_reward > 0,
                "'{}' has non-positive reward",
                next.id
            );
            assert!(
                next.distance > 0.0,
                "'{}' has non-positive distance",
                next.id
            );
        }
    }

    #[test]
    fn every_upgrade_has_a_positive_cost_and_levels() {
        let data = GameData::load().unwrap();
        for (id, upgrade) in data.upgrades.iter() {
            assert!(
                upgrade.base_cost > 0,
                "upgrade '{id}' base_cost not positive"
            );
            assert!(upgrade.max_level >= 1, "upgrade '{id}' has no levels");
        }
    }

    #[test]
    fn chassis_cost_and_slots_rise_with_order() {
        let data = GameData::load().unwrap();
        for pair in data.chassis_ordered().windows(2) {
            let (prev, next) = (pair[0], pair[1]);
            assert!(
                next.cost >= prev.cost,
                "chassis cost regresses at '{}'",
                next.id
            );
            assert!(
                next.slots >= prev.slots,
                "chassis slots regress at '{}'",
                next.id
            );
        }
    }

    #[test]
    fn mission_difficulty_is_non_decreasing_by_order() {
        let data = GameData::load().unwrap();
        let ordered = data.missions_ordered();
        for pair in ordered.windows(2) {
            let (prev, next) = (pair[0], pair[1]);
            assert!(
                next.difficulty >= prev.difficulty,
                "difficulty regresses: '{}' (order {}) is {} but earlier '{}' (order {}) is {}",
                next.id,
                next.order,
                next.difficulty,
                prev.id,
                prev.order,
                prev.difficulty,
            );
        }
    }
}
