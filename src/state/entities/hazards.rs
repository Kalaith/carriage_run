//! Hazard kinds and the runtime `Hazard` entity.

use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardKind {
    Mud,
    FallenTree,
    Rocks,
    FirePatch,
    /// A wide river crossing that drags the wheels to a long crawl.
    RiverFord,
}

impl HazardKind {
    pub fn all() -> [Self; 5] {
        [
            Self::Mud,
            Self::FallenTree,
            Self::Rocks,
            Self::FirePatch,
            Self::RiverFord,
        ]
    }

    pub(in crate::state) fn from_id(id: &str) -> Option<Self> {
        match id {
            "mud" => Some(Self::Mud),
            "fallen_tree" => Some(Self::FallenTree),
            "rocks" => Some(Self::Rocks),
            "fire_patch" => Some(Self::FirePatch),
            "river_ford" => Some(Self::RiverFord),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Mud => "Mud",
            Self::FallenTree => "Fallen Tree",
            Self::Rocks => "Rockfall",
            Self::FirePatch => "Fire Patch",
            Self::RiverFord => "River Ford",
        }
    }

    /// Short effect tag for the field guide.
    pub fn effect_tag(self) -> &'static str {
        match self {
            Self::Mud => "Slows",
            Self::FallenTree => "Blocks",
            Self::Rocks => "Impact",
            Self::FirePatch => "Burns",
            Self::RiverFord => "Wades",
        }
    }

    pub fn codex_blurb(self) -> &'static str {
        match self {
            Self::Mud => {
                "Bogs the wheels down for a moment. Brake early — speeding through flings mud and cargo."
            }
            Self::FallenTree => {
                "A heavy trunk across the lane. A direct hit costs health and cargo, so steer around it."
            }
            Self::Rocks => "Loose stones that jolt the carriage on contact. Weave past to avoid the strike.",
            Self::FirePatch => {
                "Burning ground that scorches the carriage every moment you linger. Cross it fast."
            }
            Self::RiverFord => {
                "A wide crossing that drags the wheels to a crawl far longer than mud. Line up straight and power through."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hazard {
    pub kind: HazardKind,
    pub pos: Vec2,
    pub radius: f32,
    pub size: Vec2,
    pub active: bool,
    pub triggered: bool,
}

impl Hazard {
    pub(in crate::state) fn new(kind: HazardKind, pos: Vec2) -> Self {
        let (radius, size) = match kind {
            HazardKind::Mud => (38.0, vec2(84.0, 46.0)),
            HazardKind::FallenTree => (0.0, vec2(190.0, 36.0)),
            HazardKind::Rocks => (26.0, vec2(54.0, 42.0)),
            HazardKind::FirePatch => (34.0, vec2(74.0, 48.0)),
            HazardKind::RiverFord => (46.0, vec2(150.0, 66.0)),
        };

        Self {
            kind,
            pos,
            radius,
            size,
            active: true,
            triggered: false,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x - self.size.x * 0.5,
            self.pos.y - self.size.y * 0.5,
            self.size.x,
            self.size.y,
        )
    }
}
