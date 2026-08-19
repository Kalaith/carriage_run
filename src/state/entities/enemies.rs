//! Enemy kinds and the runtime `Enemy` entity.

use macroquad::prelude::*;
use macroquad_toolkit::timing::Timer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Wolf,
    Bandit,
    BanditArcher,
    Skeleton,
    Necromancer,
    /// Elite pack leader: a bigger, tougher, faster charging wolf.
    AlphaWolf,
    /// Elite raider: a high-durability bruiser that soaks hits and does not
    /// flee. Crossbow guards deal an authored bonus against it.
    ArmoredBandit,
    /// Act II bruiser that pounds the carriage instead of fleeing.
    Ogre,
    /// Act II mounted charger that can bypass a distracted escort.
    WargRider,
    /// Act II/III support caster that weakens guard armor nearby.
    Cultist,
    /// Act II/III fast threat that leaves a burning trail.
    EmberHound,
    /// Act III ranged spirit that phases through the first order it sees.
    FrostWraith,
}

impl EnemyKind {
    pub fn all() -> [Self; 12] {
        [
            Self::Wolf,
            Self::Bandit,
            Self::BanditArcher,
            Self::Skeleton,
            Self::Necromancer,
            Self::AlphaWolf,
            Self::ArmoredBandit,
            Self::Ogre,
            Self::WargRider,
            Self::Cultist,
            Self::EmberHound,
            Self::FrostWraith,
        ]
    }

    pub(in crate::state) fn from_id(id: &str) -> Option<Self> {
        match id {
            "wolf" => Some(Self::Wolf),
            "bandit" => Some(Self::Bandit),
            "bandit_archer" => Some(Self::BanditArcher),
            "skeleton" => Some(Self::Skeleton),
            "necromancer" => Some(Self::Necromancer),
            "alpha_wolf" => Some(Self::AlphaWolf),
            "armored_bandit" => Some(Self::ArmoredBandit),
            "ogre" => Some(Self::Ogre),
            "warg_rider" => Some(Self::WargRider),
            "cultist" => Some(Self::Cultist),
            "ember_hound" => Some(Self::EmberHound),
            "frost_wraith" => Some(Self::FrostWraith),
            _ => None,
        }
    }

    /// Display name for the field guide and readouts.
    pub fn label(self) -> &'static str {
        match self {
            Self::Wolf => "Wolf",
            Self::Bandit => "Bandit",
            Self::BanditArcher => "Bandit Archer",
            Self::Skeleton => "Skeleton",
            Self::Necromancer => "Necromancer",
            Self::AlphaWolf => "Alpha Wolf",
            Self::ArmoredBandit => "Armored Bandit",
            Self::Ogre => "Ogre",
            Self::WargRider => "Warg Rider",
            Self::Cultist => "Cultist",
            Self::EmberHound => "Ember Hound",
            Self::FrostWraith => "Frost Wraith",
        }
    }

    /// Short role tag for the field guide.
    pub fn threat_tag(self) -> &'static str {
        match self {
            Self::Wolf => "Melee charger",
            Self::Bandit => "Cargo thief",
            Self::BanditArcher => "Ranged skirmisher",
            Self::Skeleton => "Undead melee",
            Self::Necromancer => "Summoner",
            Self::AlphaWolf => "Elite charger",
            Self::ArmoredBandit => "High-durability bruiser",
            Self::Ogre => "Siege bruiser",
            Self::WargRider => "Mounted charger",
            Self::Cultist => "Hex support",
            Self::EmberHound => "Burning runner",
            Self::FrostWraith => "Phasing skirmisher",
        }
    }

    /// One- to two-sentence behaviour note and counter-play for the field guide.
    pub fn codex_blurb(self) -> &'static str {
        match self {
            Self::Wolf => {
                "Sprints at the carriage in a burst. Post a melee guard to intercept it before it lands a bite."
            }
            Self::Bandit => {
                "Grabs cargo and bolts for the map edge. Cut down the fleeing thief to recover what it stole."
            }
            Self::BanditArcher => {
                "Hangs back and looses arrows from range. Send a guard to run it down or a mounted archer to trade."
            }
            Self::Skeleton => {
                "Slow but durable undead that hits hard, and a necromancer can raise it again. Focus it down."
            }
            Self::Necromancer => {
                "Keeps its distance, kites, and raises fresh skeletons. Kill it first to stem the tide."
            }
            Self::AlphaWolf => {
                "A pack leader — bigger, faster, and far tougher than a lone wolf. Gang up on it before it reaches the carriage."
            }
            Self::ArmoredBandit => {
                "A high-health raider that grinds toward the carriage without fleeing. A 2-star Crossbow Guard deals 35% bonus damage to it."
            }
            Self::Ogre => {
                "A huge bruiser that ignores panic and batters the carriage. Keep it away from the hull with Shield Guards and focused fire."
            }
            Self::WargRider => {
                "A mounted charger that accelerates hard on the final approach. Spearmen and early drag orders are its counter."
            }
            Self::Cultist => {
                "A hexer that makes nearby enemies harder to stop. Break its support circle before the next wave lands."
            }
            Self::EmberHound => {
                "A fast hound that burns the road behind it. Brake through the fire and let mounted guards take the chase."
            }
            Self::FrostWraith => {
                "A cold spirit that phases past the first command it sees. Reissue the order and keep it out of the carriage shadow."
            }
        }
    }

    pub(in crate::state) fn attack_label(self) -> &'static str {
        match self {
            Self::Wolf => "Wolf bite",
            Self::Bandit => "Bandit stole cargo",
            Self::BanditArcher => "Bandit arrow",
            Self::Skeleton => "Skeleton strike",
            Self::Necromancer => "Dark bolt",
            Self::AlphaWolf => "Alpha wolf maul",
            Self::ArmoredBandit => "Armored bandit strike",
            Self::Ogre => "Ogre smash",
            Self::WargRider => "Warg rider charge",
            Self::Cultist => "Cultist hex",
            Self::EmberHound => "Ember hound bite",
            Self::FrostWraith => "Frost wraith touch",
        }
    }

    /// Ranged skirmishers keep their distance: if a target closes inside this
    /// range they back away instead of standing to fight. `None` = no kiting.
    pub(in crate::state) fn kite_min_range(self) -> Option<f32> {
        match self {
            Self::BanditArcher => Some(150.0),
            Self::Necromancer => Some(168.0),
            Self::Cultist | Self::FrostWraith => Some(154.0),
            _ => None,
        }
    }

    /// Melee chargers commit to a burst of speed on the final approach.
    pub(in crate::state) fn charge_multiplier(self) -> f32 {
        match self {
            Self::Wolf => 1.85,
            Self::AlphaWolf => 2.05,
            Self::WargRider => 2.2,
            Self::EmberHound => 2.35,
            _ => 1.0,
        }
    }

    pub(in crate::state) fn is_charger(self) -> bool {
        matches!(
            self,
            Self::Wolf | Self::AlphaWolf | Self::WargRider | Self::EmberHound
        )
    }

    /// Bandits grab cargo and run for the map edge; killing a fleeing thief
    /// recovers what it stole.
    pub(in crate::state) fn steals_and_flees(self) -> bool {
        matches!(self, Self::Bandit)
    }
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub id: u32,
    pub kind: EnemyKind,
    pub pos: Vec2,
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub damage: f32,
    pub radius: f32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub cooldown: f32,
    pub special_timer: f32,
    pub slow_timer: f32,
    pub hit_flash: Timer,
    /// Cargo a fleeing bandit is carrying off; recovered if it is killed.
    pub carried_cargo: f32,
    /// True once a thief has stolen cargo and is running for the map edge.
    pub retreating: bool,
    pub animation_time: f32,
}

impl Enemy {
    pub(in crate::state) fn new(id: u32, kind: EnemyKind, pos: Vec2, difficulty: f32) -> Self {
        let (health, speed, damage, radius, cooldown) = match kind {
            EnemyKind::Wolf => (32.0, 124.0, 7.0, 18.0, 0.85),
            EnemyKind::Bandit => (44.0, 92.0, 5.0, 20.0, 1.05),
            EnemyKind::BanditArcher => (34.0, 62.0, 6.0, 18.0, 1.45),
            EnemyKind::Skeleton => (54.0, 76.0, 8.0, 21.0, 1.15),
            EnemyKind::Necromancer => (74.0, 48.0, 9.0, 22.0, 1.75),
            EnemyKind::AlphaWolf => (78.0, 138.0, 12.0, 24.0, 0.8),
            EnemyKind::ArmoredBandit => (96.0, 78.0, 8.0, 22.0, 1.2),
            EnemyKind::Ogre => (150.0, 52.0, 2.0, 32.0, 1.55),
            EnemyKind::WargRider => (82.0, 132.0, 2.0, 24.0, 0.88),
            EnemyKind::Cultist => (58.0, 56.0, 1.0, 19.0, 1.55),
            EnemyKind::EmberHound => (44.0, 154.0, 1.5, 19.0, 0.72),
            EnemyKind::FrostWraith => (76.0, 70.0, 2.0, 21.0, 1.28),
        };
        let scale = 0.9 + difficulty * 0.16;

        Self {
            id,
            kind,
            pos,
            health: health * scale,
            max_health: health * scale,
            speed: speed * (0.95 + difficulty * 0.04),
            damage: damage * (0.92 + difficulty * 0.08),
            radius,
            attack_range: match kind {
                EnemyKind::BanditArcher => 235.0,
                EnemyKind::Necromancer => 205.0,
                EnemyKind::Cultist | EnemyKind::FrostWraith => 190.0,
                _ => radius + 32.0,
            },
            attack_cooldown: cooldown,
            cooldown: self_stagger(id),
            special_timer: 2.4,
            slow_timer: 0.0,
            hit_flash: Timer::new(0.0),
            carried_cargo: 0.0,
            retreating: false,
            animation_time: 0.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.health > 0.0
    }

    pub(in crate::state) fn guard_aggro_range(&self) -> f32 {
        match self.kind {
            EnemyKind::Bandit => 62.0,
            EnemyKind::BanditArcher => 170.0,
            EnemyKind::Wolf => 104.0,
            EnemyKind::Skeleton => 96.0,
            EnemyKind::Necromancer => 150.0,
            EnemyKind::AlphaWolf => 122.0,
            EnemyKind::ArmoredBandit => 72.0,
            EnemyKind::Ogre => 110.0,
            EnemyKind::WargRider => 126.0,
            EnemyKind::Cultist => 146.0,
            EnemyKind::EmberHound => 112.0,
            EnemyKind::FrostWraith => 142.0,
        }
    }
}

fn self_stagger(id: u32) -> f32 {
    (id % 5) as f32 * 0.08
}
