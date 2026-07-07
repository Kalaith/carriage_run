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

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub missions: DataRegistry<MissionDef>,
    pub upgrades: DataRegistry<UpgradeDef>,
    pub chassis: DataRegistry<ChassisDef>,
    pub texture_manifest: Vec<TextureConfig>,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let config = load_embedded_json_labeled("game_config", GAME_CONFIG_JSON)?;
        let missions = DataRegistry::from_embedded_json(MISSIONS_JSON, "id")?;
        let upgrades = DataRegistry::from_embedded_json(UPGRADES_JSON, "id")?;
        let chassis = DataRegistry::from_embedded_json(CARRIAGES_JSON, "id")?;
        let texture_manifest = load_embedded_json(TEXTURE_MANIFEST_JSON)?;

        Ok(Self {
            config,
            missions,
            upgrades,
            chassis,
            texture_manifest,
        })
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
