//! Save slot reads/writes and campaign (re)initialisation.

use super::Game;
use crate::state::{migrate_save_value, GameSession, SaveData};
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
};

impl Game {
    pub(super) fn start_new_campaign(&mut self) {
        self.session = GameSession::new(&self.data.config, self.data.first_mission_id());
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
        if !self.session.campaign.auto_save_enabled {
            return;
        }

        if let Err(err) = self.write_save() {
            self.notifications
                .warning(format!("Autosave failed: {}", err));
        }
    }

    pub(super) fn write_save(&mut self) -> Result<(), String> {
        let save = self.session.to_save(&self.data.config.version);
        save_to_slot_with_version(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &save,
            &self.data.config.version,
        )?;
        self.refresh_save_state();
        Ok(())
    }

    /// Read and migrate the save slot without applying it — used both to load
    /// and to check up front whether a save is actually usable.
    pub(super) fn try_load_save(&self) -> Result<SaveData, String> {
        let first_mission = self.data.first_mission_id().map(ToOwned::to_owned);
        load_from_slot_with_migration(
            &self.data.config.game_name,
            &self.data.config.save_slot,
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

    pub(super) fn refresh_save_state(&mut self) {
        self.save_exists = slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
    }
}
