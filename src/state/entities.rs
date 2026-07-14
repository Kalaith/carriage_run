//! Mission entities and small geometry helpers.

use macroquad::prelude::*;

pub const PLAY_WIDTH: f32 = 1280.0;
pub const PLAY_TOP: f32 = 96.0;
pub const PLAY_BOTTOM: f32 = 596.0;
pub const ROAD_LEFT: f32 = 286.0;
pub const ROAD_RIGHT: f32 = 994.0;
pub const ROAD_WIDTH: f32 = ROAD_RIGHT - ROAD_LEFT;
pub const ROAD_CENTER: f32 = (ROAD_LEFT + ROAD_RIGHT) * 0.5;
pub const CARRIAGE_Y: f32 = 506.0;

pub fn road_curve_offset(y: f32, progress: f32) -> f32 {
    let primary = (progress * 0.020 + y * 0.010).sin() * 58.0;
    let secondary = (progress * 0.012 - y * 0.018).sin() * 28.0;
    (primary + secondary).clamp(-92.0, 92.0)
}

pub fn road_center_at_y(y: f32, progress: f32) -> f32 {
    ROAD_CENTER + road_curve_offset(y, progress)
}

pub fn road_left_at_y(y: f32, progress: f32) -> f32 {
    road_center_at_y(y, progress) - ROAD_WIDTH * 0.5
}

pub fn road_right_at_y(y: f32, progress: f32) -> f32 {
    road_center_at_y(y, progress) + ROAD_WIDTH * 0.5
}

pub fn clamp_x_to_road_at_y(x: f32, y: f32, progress: f32, margin: f32) -> f32 {
    x.clamp(
        road_left_at_y(y, progress) + margin,
        road_right_at_y(y, progress) - margin,
    )
}

#[derive(Debug, Clone)]
pub struct Carriage {
    pub pos: Vec2,
    pub target_x: f32,
    pub health: f32,
    pub max_health: f32,
    pub cargo: f32,
    pub max_cargo: f32,
    pub slow_timer: f32,
    pub hit_flash: f32,
}

impl Carriage {
    pub(super) fn new(max_health: f32, max_cargo: f32) -> Self {
        Self {
            pos: vec2(ROAD_CENTER, CARRIAGE_Y),
            target_x: ROAD_CENTER,
            health: max_health,
            max_health,
            cargo: max_cargo,
            max_cargo,
            slow_timer: 0.0,
            hit_flash: 0.0,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x - 40.0, self.pos.y - 50.0, 80.0, 96.0)
    }

    pub(super) fn update_timers(&mut self, dt: f32) {
        self.slow_timer = (self.slow_timer - dt).max(0.0);
        self.hit_flash = (self.hit_flash - dt).max(0.0);
    }
}

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

    pub(super) fn threat_bonus(self, stars: u8) -> f32 {
        match self {
            Self::ShieldGuard => 54.0 + stars.saturating_sub(1) as f32 * 18.0,
            Self::Spearman => 18.0,
            Self::Swordsman => 0.0,
            Self::Archer | Self::CrossbowGuard | Self::Mage => 8.0,
        }
    }

    pub(super) fn shot_color(self) -> Color {
        match self {
            Self::Archer => Color::new(0.95, 0.82, 0.38, 1.0),
            Self::CrossbowGuard => Color::new(0.82, 0.86, 0.90, 1.0),
            Self::Mage => Color::new(0.58, 0.86, 1.0, 1.0),
            _ => Color::new(0.95, 0.76, 0.28, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GuardProfile {
    pub max_health: f32,
    pub attack: f32,
    pub attack_cooldown: f32,
    pub speed: f32,
    pub range: f32,
    pub armor: f32,
}

impl GuardProfile {
    pub(super) fn new(kind: GuardKind, training_level: u32, ranged_level: u32, stars: u8) -> Self {
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
    pub hit_flash: f32,
    pub attack_flash: f32,
}

impl Guard {
    pub(super) fn new(
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
            hit_flash: 0.0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Wolf,
    Bandit,
    BanditArcher,
    Skeleton,
    Necromancer,
    /// Elite pack leader: a bigger, tougher, faster charging wolf.
    AlphaWolf,
}

impl EnemyKind {
    pub fn all() -> [Self; 6] {
        [
            Self::Wolf,
            Self::Bandit,
            Self::BanditArcher,
            Self::Skeleton,
            Self::Necromancer,
            Self::AlphaWolf,
        ]
    }

    pub(super) fn from_id(id: &str) -> Option<Self> {
        match id {
            "wolf" => Some(Self::Wolf),
            "bandit" => Some(Self::Bandit),
            "bandit_archer" => Some(Self::BanditArcher),
            "skeleton" => Some(Self::Skeleton),
            "necromancer" => Some(Self::Necromancer),
            "alpha_wolf" => Some(Self::AlphaWolf),
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
        }
    }

    pub(super) fn attack_label(self) -> &'static str {
        match self {
            Self::Wolf => "Wolf bite",
            Self::Bandit => "Bandit stole cargo",
            Self::BanditArcher => "Bandit arrow",
            Self::Skeleton => "Skeleton strike",
            Self::Necromancer => "Dark bolt",
            Self::AlphaWolf => "Alpha wolf maul",
        }
    }

    /// Ranged skirmishers keep their distance: if a target closes inside this
    /// range they back away instead of standing to fight. `None` = no kiting.
    pub(super) fn kite_min_range(self) -> Option<f32> {
        match self {
            Self::BanditArcher => Some(150.0),
            Self::Necromancer => Some(168.0),
            _ => None,
        }
    }

    /// Melee chargers commit to a burst of speed on the final approach.
    pub(super) fn charge_multiplier(self) -> f32 {
        match self {
            Self::Wolf => 1.85,
            Self::AlphaWolf => 2.05,
            _ => 1.0,
        }
    }

    /// Bandits grab cargo and run for the map edge; killing a fleeing thief
    /// recovers what it stole.
    pub(super) fn steals_and_flees(self) -> bool {
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
    pub hit_flash: f32,
    /// Cargo a fleeing bandit is carrying off; recovered if it is killed.
    pub carried_cargo: f32,
    /// True once a thief has stolen cargo and is running for the map edge.
    pub retreating: bool,
}

impl Enemy {
    pub(super) fn new(id: u32, kind: EnemyKind, pos: Vec2, difficulty: f32) -> Self {
        let (health, speed, damage, radius, cooldown) = match kind {
            EnemyKind::Wolf => (32.0, 124.0, 7.0, 18.0, 0.85),
            EnemyKind::Bandit => (44.0, 92.0, 5.0, 20.0, 1.05),
            EnemyKind::BanditArcher => (34.0, 62.0, 6.0, 18.0, 1.45),
            EnemyKind::Skeleton => (54.0, 76.0, 8.0, 21.0, 1.15),
            EnemyKind::Necromancer => (74.0, 48.0, 9.0, 22.0, 1.75),
            EnemyKind::AlphaWolf => (78.0, 138.0, 12.0, 24.0, 0.8),
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
                _ => radius + 32.0,
            },
            attack_cooldown: cooldown,
            cooldown: self_stagger(id),
            special_timer: 2.4,
            slow_timer: 0.0,
            hit_flash: 0.0,
            carried_cargo: 0.0,
            retreating: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.health > 0.0
    }

    pub(super) fn guard_aggro_range(&self) -> f32 {
        match self.kind {
            EnemyKind::Bandit => 62.0,
            EnemyKind::BanditArcher => 170.0,
            EnemyKind::Wolf => 104.0,
            EnemyKind::Skeleton => 96.0,
            EnemyKind::Necromancer => 150.0,
            EnemyKind::AlphaWolf => 122.0,
        }
    }
}

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

    pub(super) fn from_id(id: &str) -> Option<Self> {
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
    pub(super) fn new(kind: HazardKind, pos: Vec2) -> Self {
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

#[derive(Debug, Clone)]
pub struct Shot {
    pub from: Vec2,
    pub to: Vec2,
    pub timer: f32,
    pub total: f32,
    pub color: Color,
}

impl Shot {
    pub(super) fn new(from: Vec2, to: Vec2, color: Color) -> Self {
        Self {
            from,
            to,
            timer: 0.18,
            total: 0.18,
            color,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DragState {
    None,
    Carriage,
    Guard { guard_id: u32, grab: Vec2 },
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub text: String,
    pub timer: f32,
}

impl Alert {
    pub(super) fn set(&mut self, text: &str) {
        self.text = text.to_owned();
        self.timer = 1.6;
    }

    pub(super) fn update(&mut self, dt: f32) {
        self.timer = (self.timer - dt).max(0.0);
    }
}

impl Default for Alert {
    fn default() -> Self {
        Self {
            text: String::new(),
            timer: 0.0,
        }
    }
}

pub(super) fn contains_point(rect: Rect, point: Vec2) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.w
        && point.y >= rect.y
        && point.y <= rect.y + rect.h
}

pub(super) fn clamp_to_field(point: Vec2) -> Vec2 {
    vec2(
        point.x.clamp(ROAD_LEFT - 90.0, ROAD_RIGHT + 90.0),
        point.y.clamp(PLAY_TOP + 25.0, PLAY_BOTTOM - 25.0),
    )
}

pub(super) fn move_towards(current: Vec2, target: Vec2, max_delta: f32) -> Vec2 {
    let delta = target - current;
    let dist = delta.length();
    if dist <= max_delta || dist <= f32::EPSILON {
        target
    } else {
        current + delta / dist * max_delta
    }
}

fn self_stagger(id: u32) -> f32 {
    (id % 5) as f32 * 0.08
}
