//! Save slot reads/writes and campaign (re)initialisation.

use super::Game;
use crate::state::{migrate_save_value, GameSession, SaveData};
use macroquad_toolkit::persistence::{
    delete_slot, get_save_slots, load_from_slot, load_from_slot_with_migration,
    save_to_slot_with_version, slot_exists,
};

impl Game {
    pub(super) fn start_new_campaign(&mut self) {
        let active_slot = self.session.campaign.active_save_slot.clone();
        self.session = GameSession::new(&self.data.config, self.data.first_mission_id());
        self.session.campaign.active_save_slot = active_slot;
        self.session.sync_chassis(&self.data);
        self.session.open_map();
        self.notifications.info("New caravan charter started");
        self.auto_save();
    }

    pub(super) fn save_game(&mut self) {
        match self.write_save() {
            Ok(()) => self.notifications.success("Campaign saved"),
            Err(err) => self.notifications.danger(format!("Save failed: {}", err)),
        }
    }

    pub(super) fn auto_save(&mut self) {
        self.save_dirty = true;
    }

    pub(super) fn write_save(&mut self) -> Result<(), String> {
        let save = self.session.to_save(&self.data.config.version);
        let slot = self.session.campaign.active_save_slot.clone();
        self.rotate_backups(&slot);
        save_to_slot_with_version(
            &self.data.config.game_name,
            &slot,
            &save,
            &self.data.config.version,
        )?;
        self.save_dirty = false;
        self.autosave.reset_timer();
        self.refresh_save_state();
        Ok(())
    }

    fn rotate_backups(&self, slot: &str) {
        for index in (2..=3).rev() {
            let old = format!("{slot}_backup_{}", index - 1);
            let next = format!("{slot}_backup_{index}");
            if let Ok(save) = load_from_slot::<SaveData>(&self.data.config.game_name, &old) {
                let _ = save_to_slot_with_version(
                    &self.data.config.game_name,
                    &next,
                    &save,
                    &self.data.config.version,
                );
            }
        }
        if let Ok(save) = load_from_slot::<SaveData>(&self.data.config.game_name, slot) {
            let _ = save_to_slot_with_version(
                &self.data.config.game_name,
                &format!("{slot}_backup_1"),
                &save,
                &self.data.config.version,
            );
        }
    }

    /// Read and migrate the save slot without applying it — used both to load
    /// and to check up front whether a save is actually usable.
    pub(super) fn try_load_save(&self) -> Result<SaveData, String> {
        self.try_load_save_from_slot(&self.session.campaign.active_save_slot)
    }

    pub(super) fn try_load_save_from_slot(&self, slot: &str) -> Result<SaveData, String> {
        let first_mission = self.data.first_mission_id().map(ToOwned::to_owned);
        load_from_slot_with_migration(
            &self.data.config.game_name,
            slot,
            &self.data.config.version,
            |version, value| {
                migrate_save_value(version, value, &self.data.config, first_mission.as_deref())
            },
        )
    }

    pub(super) fn load_game(&mut self) {
        match self.try_load_save() {
            Ok(save) => {
                self.session = GameSession::from_save(save, self.data.first_mission_id());
                self.session.sync_chassis(&self.data);
                self.notifications.success("Campaign loaded");
                self.refresh_save_state();
            }
            Err(err) => self.notifications.warning(format!("Load failed: {}", err)),
        }
    }

    pub(super) fn select_save_slot(&mut self, slot: &str) -> Result<(), String> {
        let slot = clean_slot_name(slot)?;
        if slot_exists(&self.data.config.game_name, &slot) {
            let save = self.try_load_save_from_slot(&slot)?;
            self.session = GameSession::from_save(save, self.data.first_mission_id());
            self.session.sync_chassis(&self.data);
        } else {
            self.session.campaign.active_save_slot = slot;
        }
        self.refresh_save_state();
        Ok(())
    }

    pub(super) fn create_save_slot(&mut self, slot: &str) -> Result<(), String> {
        let slot = clean_slot_name(slot)?;
        if slot_exists(&self.data.config.game_name, &slot) {
            return Err("save slot already exists".to_owned());
        }
        self.session.campaign.active_save_slot = slot;
        self.write_save()
    }

    pub(super) fn rename_active_save_slot(&mut self, slot: &str) -> Result<(), String> {
        let new_slot = clean_slot_name(slot)?;
        let old_slot = self.session.campaign.active_save_slot.clone();
        if new_slot == old_slot || slot_exists(&self.data.config.game_name, &new_slot) {
            return Err("save slot name is unavailable".to_owned());
        }
        self.session.campaign.active_save_slot = new_slot.clone();
        self.write_save()?;
        delete_slot(&self.data.config.game_name, &old_slot)?;
        for suffix in ["_backup_1", "_backup_2", "_backup_3"] {
            let _ = delete_slot(&self.data.config.game_name, &format!("{old_slot}{suffix}"));
        }
        Ok(())
    }

    pub(super) fn delete_active_save_slot(&mut self) -> Result<(), String> {
        let old_slot = self.session.campaign.active_save_slot.clone();
        delete_slot(&self.data.config.game_name, &old_slot)?;
        for suffix in ["_backup_1", "_backup_2", "_backup_3"] {
            let _ = delete_slot(&self.data.config.game_name, &format!("{old_slot}{suffix}"));
        }
        self.session.campaign.active_save_slot = self.data.config.save_slot.clone();
        self.save_exists = false;
        Ok(())
    }

    pub(super) fn refresh_save_state(&mut self) {
        self.save_exists = slot_exists(
            &self.data.config.game_name,
            &self.session.campaign.active_save_slot,
        );
        self.save_slots = self.available_save_slots();
    }

    pub(super) fn available_save_slots(&self) -> Vec<String> {
        let mut slots: Vec<_> = get_save_slots(&self.data.config.game_name)
            .into_iter()
            .filter(|slot| !slot.contains("_backup_") && !slot.ends_with("_corrupt"))
            .collect();
        if !slots
            .iter()
            .any(|slot| slot == &self.session.campaign.active_save_slot)
        {
            slots.push(self.session.campaign.active_save_slot.clone());
        }
        slots.sort();
        slots.dedup();
        slots
    }
}

fn clean_slot_name(slot: &str) -> Result<String, String> {
    let cleaned: String = slot
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '-' || *character == '_'
        })
        .take(24)
        .collect();
    if cleaned.is_empty() || cleaned.contains("backup") || cleaned.ends_with("corrupt") {
        Err("save slot names use letters, numbers, '-' or '_'".to_owned())
    } else {
        Ok(cleaned)
    }
}
