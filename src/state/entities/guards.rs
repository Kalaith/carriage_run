//! Guard kinds, stat profiles, and the runtime `Guard` entity.

use macroquad::prelude::*;
use macroquad_toolkit::timing::Timer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardKind {
    Swordsman,
    ShieldGuard,
    Spearman,
    Archer,
    CrossbowGuard,
    Mage,
}

impl GuardKind {
    pub fn all() -> [Self; 6] {
        [
            Self::Swordsman,
            Self::ShieldGuard,
            Self::Spearman,
            Self::Archer,
            Self::CrossbowGuard,
            Self::Mage,
        ]
    }

    pub fn melee_all() -> [Self; 3] {
        [Self::Swordsman, Self::ShieldGuard, Self::Spearman]
    }

    pub fn ranged_all() -> [Self; 3] {
        [Self::Archer, Self::CrossbowGuard, Self::Mage]
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "shield_guard" => Self::ShieldGuard,
            "spearman" => Self::Spearman,
            "archer" => Self::Archer,
            "crossbow_guard" => Self::CrossbowGuard,
            "mage" => Self::Mage,
            _ => Self::Swordsman,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Swordsman => "swordsman",
            Self::ShieldGuard => "shield_guard",
            Self::Spearman => "spearman",
            Self::Archer => "archer",
            Self::CrossbowGuard => "crossbow_guard",
            Self::Mage => "mage",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Swordsman => "Swordsman",
            Self::ShieldGuard => "Shield Guard",
            Self::Spearman => "Spearman",
            Self::Archer => "Archer",
            Self::CrossbowGuard => "Crossbow Guard",
            Self::Mage => "Mage",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Swordsman => "Balanced melee guard with reliable damage and speed.",
            Self::ShieldGuard => "Tough defender that pulls enemies off the carriage.",
            Self::Spearman => "Longer reach and extra damage against charging wolves.",
            Self::Archer => {
                "Fast ranged guard that can mount the carriage and pick off weak enemies."
            }
            Self::CrossbowGuard => {
                "Slow ranged guard with heavy bolts that punish armored targets."
            }
            Self::Mage => "Arcane ranged guard with splash damage and late-star support magic.",
        }
    }

    pub fn unlock_level(self) -> u32 {
        match self {
            Self::Swordsman => 1,
            Self::ShieldGuard => 2,
            Self::Spearman => 3,
            Self::Archer => 1,
            Self::CrossbowGuard => 2,
            Self::Mage => 3,
        }
    }

    pub fn hire_cost(self) -> i64 {
        match self {
            Self::Swordsman => 0,
            Self::ShieldGuard => 120,
            Self::Spearman => 170,
            Self::Archer => 0,
            Self::CrossbowGuard => 135,
            Self::Mage => 190,
        }
    }

    pub fn is_ranged(self) -> bool {
        matches!(self, Self::Archer | Self::CrossbowGuard | Self::Mage)
    }

    pub fn is_melee(self) -> bool {
        !self.is_ranged()
    }

    pub fn star_upgrade_cost(self, current_stars: u8) -> Option<i64> {
        match current_stars {
            1 => Some(match self {
                Self::Swordsman | Self::Archer => 90,
                Self::ShieldGuard | Self::CrossbowGuard => 120,
                Self::Spearman | Self::Mage => 145,
            }),
            2 => Some(match self {
                Self::Swordsman | Self::Archer => 160,
                Self::ShieldGuard | Self::CrossbowGuard => 205,
                Self::Spearman | Self::Mage => 240,
            }),
            _ => None,
        }
    }

    pub fn ability_summary(self, stars: u8) -> &'static str {
        match (self, stars) {
            (Self::Swordsman, 1) => "1*: Reliable melee attacks.",
            (Self::Swordsman, 2) => "2*: Faster sword recovery.",
            (Self::Swordsman, _) => "3*: Cleave damages a second nearby enemy.",
            (Self::ShieldGuard, 1) => "1*: High armor and threat.",
            (Self::ShieldGuard, 2) => "2*: Stronger threat pull.",
            (Self::ShieldGuard, _) => "3*: Shield Wall reduces nearby carriage damage.",
            (Self::Spearman, 1) => "1*: Long reach, good against wolves.",
            (Self::Spearman, 2) => "2*: Wider brace range.",
            (Self::Spearman, _) => "3*: Heavy brace damage against charging enemies.",
            (Self::Archer, 1) => "1*: Quick mounted arrows.",
            (Self::Archer, 2) => "2*: Quicker reload.",
            (Self::Archer, _) => "3*: Piercing shot can hit a second target.",
            (Self::CrossbowGuard, 1) => "1*: Slow heavy bolts.",
            (Self::CrossbowGuard, 2) => "2*: Armor-piercing undead bolts.",
            (Self::CrossbowGuard, _) => "3*: Pinning shots slow targets.",
            (Self::Mage, 1) => "1*: Magic bolts ignore some armor.",
            (Self::Mage, 2) => "2*: Splash damage around the target.",
            (Self::Mage, _) => "3*: Healing charm restores an injured guard.",
        }
    }

    pub fn stat_summary(self, training_level: u32, ranged_level: u32, stars: u8) -> String {
        let profile = GuardProfile::new(self, training_level, ranged_level, stars);

        format!(
            "HP {:.0} | ATK {:.0} | SPD {:.0} | RNG {:.0} | ARM {:.1}",
            profile.max_health, profile.attack, profile.speed, profile.range, profile.armor
        )
    }

    pub(in crate::state) fn threat_bonus(self, stars: u8) -> f32 {
        match self {
            Self::ShieldGuard => 54.0 + stars.saturating_sub(1) as f32 * 18.0,
            Self::Spearman => 18.0,
            Self::Swordsman => 0.0,
            Self::Archer | Self::CrossbowGuard | Self::Mage => 8.0,
        }
    }

    pub(in crate::state) fn shot_color(self) -> Color {
        match self {
            Self::Archer => Color::new(0.95, 0.82, 0.38, 1.0),
            Self::CrossbowGuard => Color::new(0.82, 0.86, 0.90, 1.0),
            Self::Mage => Color::new(0.58, 0.86, 1.0, 1.0),
            _ => Color::new(0.95, 0.76, 0.28, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(in crate::state) struct GuardProfile {
    pub max_health: f32,
    pub attack: f32,
    pub attack_cooldown: f32,
    pub speed: f32,
    pub range: f32,
    pub armor: f32,
}

impl GuardProfile {
    pub(in crate::state) fn new(
        kind: GuardKind,
        training_level: u32,
        ranged_level: u32,
        stars: u8,
    ) -> Self {
        let level = if kind.is_ranged() {
            ranged_level as f32
        } else {
            training_level as f32
        };
        let star_bonus = stars.saturating_sub(1) as f32;
        let (max_health, attack, cooldown, speed, range, armor) = match kind {
            GuardKind::Swordsman => (
                64.0 + level * 18.0,
                13.0 + level * 4.0,
                0.62,
                150.0 + level * 8.0,
                34.0,
                1.5 + level * 0.45,
            ),
            GuardKind::ShieldGuard => (
                92.0 + level * 23.0,
                9.0 + level * 3.0,
                0.78,
                118.0 + level * 5.0,
                32.0,
                5.0 + level * 0.9,
            ),
            GuardKind::Spearman => (
                60.0 + level * 16.0,
                12.0 + level * 3.5,
                0.68,
                142.0 + level * 8.0,
                54.0 + star_bonus * 5.0,
                1.0 + level * 0.35,
            ),
            GuardKind::Archer => (
                48.0 + level * 11.0,
                10.0 + level * 3.5,
                0.86 - star_bonus * 0.08,
                132.0 + level * 6.0,
                250.0 + level * 18.0,
                0.8 + level * 0.25,
            ),
            GuardKind::CrossbowGuard => (
                56.0 + level * 13.0,
                18.0 + level * 4.5,
                1.34 - star_bonus * 0.06,
                112.0 + level * 5.0,
                285.0 + level * 14.0,
                1.2 + level * 0.3,
            ),
            GuardKind::Mage => (
                46.0 + level * 10.0,
                14.0 + level * 4.0,
                1.08 - star_bonus * 0.05,
                118.0 + level * 5.0,
                260.0 + level * 16.0,
                0.6 + level * 0.2,
            ),
        };

        Self {
            max_health: max_health * (1.0 + star_bonus * 0.16),
            attack: attack * (1.0 + star_bonus * 0.15),
            attack_cooldown: cooldown.max(0.42),
            speed: speed * (1.0 + star_bonus * 0.05),
            range: range * (1.0 + star_bonus * 0.04),
            armor: armor + star_bonus * 0.55,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Guard {
    pub id: u32,
    pub kind: GuardKind,
    pub pos: Vec2,
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub attack_cooldown: f32,
    pub speed: f32,
    pub range: f32,
    pub armor: f32,
    pub order: GuardOrder,
    pub star_level: u8,
    pub mounted_slot: Option<usize>,
    pub cooldown: f32,
    pub hit_flash: Timer,
    pub attack_flash: f32,
}

impl Guard {
    pub(in crate::state) fn new(
        id: u32,
        kind: GuardKind,
        pos: Vec2,
        training_level: u32,
        ranged_level: u32,
        star_level: u8,
        mounted_slot: Option<usize>,
    ) -> Self {
        let profile = GuardProfile::new(kind, training_level, ranged_level, star_level);

        Self {
            id,
            kind,
            pos,
            health: profile.max_health,
            max_health: profile.max_health,
            attack: profile.attack,
            attack_cooldown: profile.attack_cooldown,
            speed: profile.speed,
            range: profile.range,
            armor: profile.armor,
            order: default_stance(kind),
            star_level,
            mounted_slot,
            cooldown: 0.0,
            hit_flash: Timer::new(0.0),
            attack_flash: 0.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.health > 0.0
    }

    /// The stance a guard falls back to when a direct order (attack/return)
    /// completes: melee guards roam, ranged guards hold formation to mount.
    pub fn home_stance(&self) -> GuardOrder {
        default_stance(self.kind)
    }

    /// Short label describing the guard's current standing order for the HUD.
    pub fn stance_label(&self) -> &'static str {
        if self.mounted_slot.is_some() {
            "Mounted"
        } else {
            match self.order {
                GuardOrder::Roam => "Roam",
                GuardOrder::Escort => "Escort",
                GuardOrder::Hold => "Hold",
                GuardOrder::Move(_) => "Move",
                GuardOrder::Attack(_) => "Attack",
            }
        }
    }
}

fn default_stance(kind: GuardKind) -> GuardOrder {
    if kind.is_melee() {
        GuardOrder::Roam
    } else {
        GuardOrder::Escort
    }
}

#[derive(Debug, Clone)]
pub enum GuardOrder {
    /// Hold formation next to the carriage, attacking only what wanders into range.
    Escort,
    /// Self-directing stance: intercept the nearest enemy inside the carriage
    /// leash radius, then fall back to formation when the area is clear.
    Roam,
    Move(Vec2),
    Hold,
    Attack(u32),
}

/// Radius around the carriage inside which a roaming guard will chase enemies
/// before returning to escort formation.
pub const ROAM_LEASH_RADIUS: f32 = 232.0;
