//! Embedded game data and asset manifests.

use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::{
    load_embedded_json, load_embedded_json_labeled, DataRegistry,
};
use serde::{Deserialize, Serialize};

const GAME_CONFIG_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/game_config.json");
const MISSIONS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/missions.json");
const EXTENDED_MISSIONS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/missions_act2_act3.json");
const UPGRADES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/upgrades.json");
const CARRIAGES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/carriages.json");
const TEXTURE_MANIFEST_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/texture_manifest.json");
const RELICS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/relics.json");
const LEG_MODIFIERS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/leg_modifiers.json");
const RUN_EVENTS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/run_events.json");
const STAKES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/stakes.json");
const CARRIAGE_FRAMES_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/carriage_frames.json");
const GUARD_SPECIALIZATIONS_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/guard_specializations.json");
const COSMETICS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/cosmetics.json");

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_name: String,
    pub display_name: String,
    pub save_slot: String,
    pub version: String,
    #[serde(default)]
    pub toolkit_revision: String,
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
    /// Authored campaign act. Legacy missions derive this from their order.
    #[serde(default)]
    pub act: u8,
    /// Authored biome used by the route map, hazards, and art palette.
    #[serde(default)]
    pub biome: String,
    /// Optional reusable boss definition used by finale missions.
    #[serde(default)]
    pub boss_id: Option<String>,
    /// Optional content that appears after the three core acts.
    #[serde(default)]
    pub side_mission: bool,
    /// Explicit hazard palette for validation and route-map presentation.
    #[serde(default)]
    pub hazard_palette: Vec<String>,
    /// Short reward rationale shown on the results breakdown.
    #[serde(default)]
    pub reward_note: String,
}

impl MissionDef {
    pub fn authored_act(&self) -> u8 {
        if self.act > 0 {
            self.act
        } else {
            // The original twelve routes are the fully authored opening act;
            // extended act metadata is explicit in the release content file.
            1
        }
    }

    pub fn authored_biome(&self) -> &str {
        if self.biome.is_empty() {
            match self.authored_act() {
                1 => "greenwood",
                2 => "ashen_march",
                _ => "moonlit_frontier",
            }
        } else {
            self.biome.as_str()
        }
    }

    pub fn palette(&self) -> &[String] {
        &self.hazard_palette
    }
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
    /// Flat damage points removed from each carriage hit (can be negative).
    #[serde(default)]
    pub flat_armor_add: f32,
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

/// A between-legs expedition vignette: a short non-combat decision with a few
/// outcomes, each a small resource trade. Cheap content that makes an
/// expedition feel like a journey rather than a mission list. Session-only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEventDef {
    pub id: String,
    pub order: u32,
    pub prompt: String,
    pub options: Vec<RunEventOptionDef>,
}

/// One choice within a [`RunEventDef`], with its flavor payoff and effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEventOptionDef {
    pub label: String,
    /// One-line outcome shown after the choice is made.
    pub result: String,
    /// Run-banked-gold delta. Negative costs must be affordable in full.
    #[serde(default)]
    pub gold: i64,
    /// Carriage-health-ratio delta (clamped into a survivable range).
    #[serde(default)]
    pub health: f32,
    /// Relic id granted by this choice, or empty for none.
    #[serde(default)]
    pub relic: String,
}

/// An expedition entry-stake tier: an up-front ante (gold-in) paid when a run
/// begins, in exchange for a multiplier on all banked gold that run
/// (multiplier-out). A push-your-luck gold sink — the ante is forfeit if the run
/// goes badly, so bigger wagers demand deeper runs to pay off.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: u32,
    /// Gold paid up front when the expedition begins.
    #[serde(default)]
    pub cost: i64,
    /// Multiplier applied to all gold banked during the run.
    #[serde(default = "one")]
    pub reward_mult: f32,
}

/// A mutually-exclusive carriage frame tuning: a build-identity choice that
/// trades one stat axis against another (speed / health / cargo). Exactly one is
/// active at a time, so no build gets every advantage. Applied in `MissionRun::new`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarriageFrameDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: u32,
    #[serde(default = "one")]
    pub speed_mult: f32,
    #[serde(default = "one")]
    pub health_mult: f32,
    #[serde(default = "one")]
    pub cargo_mult: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardSpecializationDef {
    pub id: String,
    pub guard_id: String,
    pub name: String,
    pub description: String,
    pub cost: i64,
    #[serde(default = "one")]
    pub damage_mult: f32,
    pub ability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmeticDef {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub description: String,
    pub cost: i64,
    pub order: u32,
    pub tint: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub missions: DataRegistry<MissionDef>,
    pub upgrades: DataRegistry<UpgradeDef>,
    pub chassis: DataRegistry<ChassisDef>,
    pub relics: DataRegistry<RelicDef>,
    pub leg_modifiers: DataRegistry<LegModifierDef>,
    pub run_events: DataRegistry<RunEventDef>,
    pub stakes: DataRegistry<StakeDef>,
    pub carriage_frames: DataRegistry<CarriageFrameDef>,
    pub guard_specializations: DataRegistry<GuardSpecializationDef>,
    pub cosmetics: DataRegistry<CosmeticDef>,
    pub texture_manifest: Vec<TextureConfig>,
}

impl GameData {
    pub fn recovery(reason: &str) -> Self {
        let _ = reason;
        Self {
            config: GameConfig {
                game_name: "carriage_run".to_owned(),
                display_name: "Carriage Run".to_owned(),
                save_slot: "campaign".to_owned(),
                version: "recovery".to_owned(),
                toolkit_revision: String::new(),
                starting_gold: 0,
            },
            missions: DataRegistry::new(),
            upgrades: DataRegistry::new(),
            chassis: DataRegistry::new(),
            relics: DataRegistry::new(),
            leg_modifiers: DataRegistry::new(),
            run_events: DataRegistry::new(),
            stakes: DataRegistry::new(),
            carriage_frames: DataRegistry::new(),
            guard_specializations: DataRegistry::new(),
            cosmetics: DataRegistry::new(),
            texture_manifest: Vec::new(),
        }
    }

    pub fn load() -> Result<Self, String> {
        let config = load_embedded_json_labeled("game_config", GAME_CONFIG_JSON)?;
        let mut missions = DataRegistry::from_embedded_json(MISSIONS_JSON, "id")?;
        missions.merge(DataRegistry::from_embedded_json(
            EXTENDED_MISSIONS_JSON,
            "id",
        )?);
        let upgrades = DataRegistry::from_embedded_json(UPGRADES_JSON, "id")?;
        let chassis = DataRegistry::from_embedded_json(CARRIAGES_JSON, "id")?;
        let relics = DataRegistry::from_embedded_json(RELICS_JSON, "id")?;
        let leg_modifiers = DataRegistry::from_embedded_json(LEG_MODIFIERS_JSON, "id")?;
        let run_events = DataRegistry::from_embedded_json(RUN_EVENTS_JSON, "id")?;
        let stakes = DataRegistry::from_embedded_json(STAKES_JSON, "id")?;
        let carriage_frames = DataRegistry::from_embedded_json(CARRIAGE_FRAMES_JSON, "id")?;
        let guard_specializations =
            DataRegistry::from_embedded_json(GUARD_SPECIALIZATIONS_JSON, "id")?;
        let cosmetics = DataRegistry::from_embedded_json(COSMETICS_JSON, "id")?;
        let texture_manifest = load_embedded_json(TEXTURE_MANIFEST_JSON)?;

        Ok(Self {
            config,
            missions,
            upgrades,
            chassis,
            relics,
            leg_modifiers,
            run_events,
            stakes,
            carriage_frames,
            guard_specializations,
            cosmetics,
            texture_manifest,
        })
    }

    pub fn run_events_ordered(&self) -> Vec<&RunEventDef> {
        let mut events: Vec<_> = self.run_events.iter().map(|(_, e)| e).collect();
        events.sort_by_key(|e| e.order);
        events
    }

    pub fn stakes_ordered(&self) -> Vec<&StakeDef> {
        let mut stakes: Vec<_> = self.stakes.iter().map(|(_, s)| s).collect();
        stakes.sort_by_key(|s| s.order);
        stakes
    }

    pub fn carriage_frames_ordered(&self) -> Vec<&CarriageFrameDef> {
        let mut frames: Vec<_> = self.carriage_frames.iter().map(|(_, f)| f).collect();
        frames.sort_by_key(|f| f.order);
        frames
    }

    pub fn cosmetics_ordered(&self) -> Vec<&CosmeticDef> {
        let mut cosmetics: Vec<_> = self
            .cosmetics
            .iter()
            .map(|(_, cosmetic)| cosmetic)
            .collect();
        cosmetics.sort_by_key(|cosmetic| cosmetic.order);
        cosmetics
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
    pub fn chassis_for_legacy_level(&self, legacy_level: u32) -> String {
        let target_slots = if legacy_level >= 4 {
            4
        } else if legacy_level >= 2 {
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
mod tests;
