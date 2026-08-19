//! Save payload shape and migration of older save formats.

use super::CampaignState;
use crate::data::GameConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub campaign: CampaignState,
    #[serde(default = "unknown_save_timestamp")]
    pub saved_at: String,
}

fn unknown_save_timestamp() -> String {
    "1970-01-01T00:00:00Z".to_owned()
}

pub fn save_timestamp() -> String {
    let seconds = macroquad::miniquad::date::now().max(0.0) as i64;
    let days = seconds.div_euclid(86_400);
    let seconds_today = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds_today / 3_600,
        seconds_today / 60 % 60,
        seconds_today % 60
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
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
        if current.saved_at == unknown_save_timestamp() {
            current.saved_at = save_timestamp();
        }
        current.campaign.normalize(first_mission_id);
        return Ok(current);
    }

    if let Ok(mut campaign) = serde_json::from_value::<CampaignState>(payload.clone()) {
        campaign.normalize(first_mission_id);
        return Ok(SaveData {
            version: config.version.clone(),
            campaign,
            saved_at: save_timestamp(),
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
        saved_at: save_timestamp(),
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
