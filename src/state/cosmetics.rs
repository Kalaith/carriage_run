//! Cosmetic ownership and preview tint helpers.

use super::{CampaignState, GameSession};
use crate::data::GameData;

impl CampaignState {
    pub fn is_livery_owned(&self, id: &str) -> bool {
        self.owned_livery_ids.iter().any(|owned| owned == id)
    }

    pub fn is_guard_color_owned(&self, id: &str) -> bool {
        self.owned_guard_color_ids.iter().any(|owned| owned == id)
    }

    pub fn livery_tint(&self) -> [f32; 3] {
        match self.livery_id.as_str() {
            "livery_roadwarden" | "livery_moonlit" => [0.78, 0.88, 1.0],
            "livery_ember" => [1.0, 0.78, 0.68],
            _ => [1.0, 1.0, 1.0],
        }
    }

    pub fn guard_color_tint(&self) -> [f32; 3] {
        match self.guard_color_id.as_str() {
            "guard_color_verdant" => [0.70, 1.0, 0.78],
            "guard_color_sunset" => [1.0, 0.78, 0.60],
            _ => [1.0, 1.0, 1.0],
        }
    }
}

impl GameSession {
    pub fn buy_cosmetic(&mut self, data: &GameData, id: &str) -> bool {
        let Some(cosmetic) = data.cosmetics.get(id) else {
            return false;
        };
        let owned = match cosmetic.kind.as_str() {
            "livery" => self.campaign.is_livery_owned(id),
            "guard_color" => self.campaign.is_guard_color_owned(id),
            _ => return false,
        };
        if owned || self.campaign.gold < cosmetic.cost {
            return false;
        }
        self.campaign.gold -= cosmetic.cost;
        match cosmetic.kind.as_str() {
            "livery" => self.campaign.owned_livery_ids.push(id.to_owned()),
            "guard_color" => self.campaign.owned_guard_color_ids.push(id.to_owned()),
            _ => return false,
        }
        true
    }

    pub fn select_cosmetic(&mut self, data: &GameData, id: &str) -> bool {
        let Some(cosmetic) = data.cosmetics.get(id) else {
            return false;
        };
        match cosmetic.kind.as_str() {
            "livery" if self.campaign.is_livery_owned(id) => {
                self.campaign.livery_id = id.to_owned();
                true
            }
            "guard_color" if self.campaign.is_guard_color_owned(id) => {
                self.campaign.guard_color_id = id.to_owned();
                true
            }
            _ => false,
        }
    }
}
