//! Carriage chassis: slot derivation, purchase, switching, and legacy-save
//! migration. Split from `state.rs` to keep that file within the size limit.

use super::{CampaignState, GameSession};
use crate::data::GameData;

impl CampaignState {
    /// Guard and equipment slot count, driven by the active chassis. Falls back
    /// to the legacy carriage-level formula for saves predating chassis.
    pub fn chassis_slot_count(&self) -> usize {
        if self.chassis_slots > 0 {
            self.chassis_slots.clamp(2, 4)
        } else if self.carriage_level >= 4 {
            4
        } else if self.carriage_level >= 2 {
            3
        } else {
            2
        }
    }

    pub fn is_chassis_owned(&self, id: &str) -> bool {
        self.owned_chassis_ids.iter().any(|owned| owned == id)
    }

    /// Copy the active chassis's stats into the cached fields used by the sim
    /// and slot counts. No-op if the chassis id is unknown.
    pub fn refresh_chassis_stats(&mut self, data: &GameData) {
        if let Some(chassis) = data.chassis.get(&self.chassis_id) {
            self.chassis_slots = chassis.slots;
            self.chassis_speed_mult = chassis.speed_mult;
            self.chassis_health_mult = chassis.health_mult;
            self.normalize_equipment();
        }
    }

    /// Resolve the active carriage frame tuning into cached multipliers, falling
    /// back to the balanced Standard Frame if the id is unknown.
    pub fn refresh_frame_stats(&mut self, data: &GameData) {
        let frame = data
            .carriage_frames
            .get(&self.carriage_frame_id)
            .or_else(|| data.carriage_frames.get("standard"));
        if let Some(frame) = frame {
            self.frame_speed_mult = frame.speed_mult;
            self.frame_health_mult = frame.health_mult;
            self.frame_cargo_mult = frame.cargo_mult;
        }
    }
}

impl GameSession {
    /// Resolve the active chassis and refresh cached stats, migrating legacy
    /// saves (no chassis recorded) to a chassis matching their old slot count.
    /// Called whenever a session is created or loaded (game.rs has `data`).
    pub fn sync_chassis(&mut self, data: &GameData) {
        let default_id = data.default_chassis_id();

        if self.campaign.owned_chassis_ids.is_empty() {
            let derived = data.chassis_for_level(self.campaign.carriage_level);
            self.campaign.owned_chassis_ids = vec![derived.clone()];
            if self.campaign.chassis_id.is_empty() {
                self.campaign.chassis_id = derived;
            }
        }

        // The starter chassis is always available.
        if !self.campaign.is_chassis_owned(&default_id) {
            self.campaign
                .owned_chassis_ids
                .insert(0, default_id.clone());
        }
        // Drop any owned ids the data no longer defines.
        self.campaign
            .owned_chassis_ids
            .retain(|id| data.chassis.get(id).is_some());

        if data.chassis.get(&self.campaign.chassis_id).is_none() {
            self.campaign.chassis_id = self
                .campaign
                .owned_chassis_ids
                .first()
                .cloned()
                .unwrap_or(default_id);
        }
        self.campaign.refresh_chassis_stats(data);
        self.campaign.refresh_frame_stats(data);
    }

    pub fn buy_chassis(&mut self, data: &GameData, id: &str) -> bool {
        let Some(chassis) = data.chassis.get(id) else {
            return false;
        };
        if self.campaign.is_chassis_owned(id) || self.campaign.gold < chassis.cost {
            return false;
        }

        self.campaign.gold -= chassis.cost;
        self.campaign.owned_chassis_ids.push(id.to_owned());
        self.campaign.chassis_id = id.to_owned();
        self.campaign.refresh_chassis_stats(data);
        true
    }

    pub fn select_chassis(&mut self, data: &GameData, id: &str) -> bool {
        if !self.campaign.is_chassis_owned(id) || data.chassis.get(id).is_none() {
            return false;
        }

        self.campaign.chassis_id = id.to_owned();
        self.campaign.refresh_chassis_stats(data);
        true
    }

    /// Selects the mutually-exclusive carriage frame tuning. Returns false if the
    /// id is unknown or already selected.
    pub fn select_frame(&mut self, data: &GameData, id: &str) -> bool {
        if !data.carriage_frames.contains(id) || self.campaign.carriage_frame_id == id {
            return false;
        }
        self.campaign.carriage_frame_id = id.to_owned();
        self.campaign.refresh_frame_stats(data);
        true
    }
}
