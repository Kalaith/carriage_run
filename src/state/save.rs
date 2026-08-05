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
    let mut payload = value.get("data").cloned().unwrap_or(value);
    inject_legacy_campaign_rank(&mut payload);

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

/// Old saves coupled unlock standing to `carriage_level`. Give those saves the
/// better of their completion-derived rank and former level exactly once, then
/// deserialize that same field as the renamed Iron Plating `armor_level`.
fn inject_legacy_campaign_rank(payload: &mut Value) {
    let campaign = if payload.get("campaign").is_some() {
        payload.get_mut("campaign").expect("campaign key exists")
    } else {
        payload
    };
    let Some(object) = campaign.as_object_mut() else {
        return;
    };
    if object.contains_key("campaign_rank") {
        return;
    }
    let completed = object
        .get("records")
        .and_then(Value::as_object)
        .map(|records| {
            records
                .values()
                .filter(|record| {
                    record
                        .get("completions")
                        .and_then(Value::as_u64)
                        .unwrap_or(0)
                        > 0
                })
                .count()
        })
        .unwrap_or(0);
    let legacy_level = object
        .get("carriage_level")
        .and_then(Value::as_u64)
        .unwrap_or(1) as u32;
    let rank = CampaignState::rank_for_completed_missions(completed).max(legacy_level);
    object.insert("campaign_rank".to_owned(), Value::from(rank.min(4)));
}
