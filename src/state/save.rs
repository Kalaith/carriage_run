//! Save payload shape and migration of older save formats.

use super::CampaignState;
use crate::data::GameConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub campaign: CampaignState,
}

#[derive(Debug, Deserialize)]
struct LegacyTemplateSave {
    points: Option<i64>,
}

pub fn migrate_save_value(
    detected_version: Option<String>,
    value: Value,
    config: &GameConfig,
    first_mission_id: Option<&str>,
) -> Result<SaveData, String> {
    let payload = value.get("data").cloned().unwrap_or(value);

    if let Ok(mut current) = serde_json::from_value::<SaveData>(payload.clone()) {
        current.version = config.version.clone();
        current.campaign.normalize(first_mission_id);
        return Ok(current);
    }

    if let Ok(mut campaign) = serde_json::from_value::<CampaignState>(payload.clone()) {
        campaign.normalize(first_mission_id);
        return Ok(SaveData {
            version: config.version.clone(),
            campaign,
        });
    }

    let legacy: LegacyTemplateSave = serde_json::from_value(payload)
        .map_err(|err| format!("Unsupported save format {:?}: {}", detected_version, err))?;
    let mut campaign = CampaignState::new(config, first_mission_id);
    if let Some(points) = legacy.points {
        campaign.gold = points.max(0);
    }

    Ok(SaveData {
        version: config.version.clone(),
        campaign,
    })
}
