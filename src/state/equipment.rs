//! Carriage equipment slots and visual state.

use super::CampaignState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarriageEquipment {
    IronPlating,
    ReinforcedWheels,
    CargoStraps,
    RepairKit,
}

impl CarriageEquipment {
    pub fn from_id(id: &str) -> Self {
        match id {
            "reinforced_wheels" => Self::ReinforcedWheels,
            "cargo_straps" => Self::CargoStraps,
            "repair_kit" => Self::RepairKit,
            _ => Self::IronPlating,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::IronPlating => "carriage_armor",
            Self::ReinforcedWheels => "reinforced_wheels",
            Self::CargoStraps => "cargo_straps",
            Self::RepairKit => "repair_kit",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::IronPlating => "Iron Plating",
            Self::ReinforcedWheels => "Reinforced Wheels",
            Self::CargoStraps => "Cargo Straps",
            Self::RepairKit => "Repair Kit",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CarriageVisual {
    pub chassis_level: u32,
    pub iron_plating: bool,
    pub reinforced_wheels: bool,
    pub cargo_straps: bool,
    pub repair_kit: bool,
    pub ranged_slots: usize,
}

impl CarriageVisual {
    pub fn from_campaign(campaign: &CampaignState) -> Self {
        Self {
            chassis_level: campaign.carriage_level,
            iron_plating: campaign.is_equipment_equipped(CarriageEquipment::IronPlating),
            reinforced_wheels: campaign.is_equipment_equipped(CarriageEquipment::ReinforcedWheels),
            cargo_straps: campaign.is_equipment_equipped(CarriageEquipment::CargoStraps),
            repair_kit: campaign.is_equipment_equipped(CarriageEquipment::RepairKit),
            ranged_slots: campaign.ranged_slot_count(),
        }
    }
}

impl CampaignState {
    pub fn carriage_equipment_slot_count(&self) -> usize {
        if self.carriage_level >= 4 {
            4
        } else if self.carriage_level >= 2 {
            3
        } else {
            2
        }
    }

    pub fn equipment_level(&self, equipment: CarriageEquipment) -> u32 {
        match equipment {
            CarriageEquipment::IronPlating => self.carriage_level,
            CarriageEquipment::ReinforcedWheels => self.wheel_level,
            CarriageEquipment::CargoStraps => self.cargo_level,
            CarriageEquipment::RepairKit => self.repair_level,
        }
    }

    pub fn is_equipment_available(&self, equipment: CarriageEquipment) -> bool {
        self.equipment_level(equipment) > 0
    }

    pub fn is_equipment_equipped(&self, equipment: CarriageEquipment) -> bool {
        self.selected_equipment_ids
            .iter()
            .any(|id| id.as_str() == equipment.id())
            && self.is_equipment_available(equipment)
    }

    pub fn assign_equipment_slot(&mut self, slot: usize, id: &str) {
        let equipment = CarriageEquipment::from_id(id);
        if slot >= self.carriage_equipment_slot_count() || !self.is_equipment_available(equipment) {
            return;
        }

        self.selected_equipment_ids
            .retain(|existing| existing != equipment.id());
        if slot >= self.selected_equipment_ids.len() {
            self.selected_equipment_ids.resize(slot + 1, String::new());
        }
        self.selected_equipment_ids[slot] = equipment.id().to_owned();
        self.selected_equipment_ids
            .retain(|existing| !existing.is_empty());
    }

    pub fn clear_equipment_slot(&mut self, slot: usize) {
        if slot < self.selected_equipment_ids.len() {
            self.selected_equipment_ids.remove(slot);
        }
    }

    pub fn normalize_equipment(&mut self) {
        let slot_count = self.carriage_equipment_slot_count();
        let mut selected = Vec::new();
        for id in self
            .selected_equipment_ids
            .iter()
            .chain(default_selected_equipment_ids().iter())
        {
            let equipment = CarriageEquipment::from_id(id);
            if !self.is_equipment_available(equipment) {
                continue;
            }
            if selected
                .iter()
                .any(|selected_id: &String| selected_id == equipment.id())
            {
                continue;
            }
            selected.push(equipment.id().to_owned());
            if selected.len() >= slot_count {
                break;
            }
        }
        self.selected_equipment_ids = selected;
    }
}

pub fn default_selected_equipment_ids() -> Vec<String> {
    vec![CarriageEquipment::IronPlating.id().to_owned()]
}
