//! Reusable multi-phase finale encounter state.

use macroquad::prelude::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossPhase {
    Approach,
    First,
    Second,
    Enraged,
    Defeated,
}

impl BossPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Approach => "Telegraph",
            Self::First => "Phase I",
            Self::Second => "Phase II",
            Self::Enraged => "Enraged",
            Self::Defeated => "Defeated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BossDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub max_health: f32,
    pub phase_two_at: f32,
    pub enraged_at: f32,
    pub telegraph_seconds: f32,
    pub attack_damage: f32,
}

impl BossDefinition {
    pub fn for_id(id: &str) -> Self {
        match id {
            "road_warden" => Self {
                id: "road_warden",
                name: "The Road Warden",
                max_health: 420.0,
                phase_two_at: 0.68,
                enraged_at: 0.30,
                telegraph_seconds: 2.0,
                attack_damage: 19.0,
            },
            "expedition_warden" => Self {
                id: "expedition_warden",
                name: "The Expedition Warden",
                max_health: 520.0,
                phase_two_at: 0.66,
                enraged_at: 0.28,
                telegraph_seconds: 1.7,
                attack_damage: 8.0,
            },
            _ => Self {
                id: "ash_colossus",
                name: "The Ash Colossus",
                max_health: 360.0,
                phase_two_at: 0.70,
                enraged_at: 0.32,
                telegraph_seconds: 2.2,
                attack_damage: 17.0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BossEvent {
    Telegraph { phase: BossPhase, target: Vec2 },
    PhaseChanged(BossPhase),
    Attack { damage: f32, target: Vec2 },
    Victory,
}

#[derive(Debug, Clone)]
pub struct BossState {
    pub definition: BossDefinition,
    pub health: f32,
    pub phase: BossPhase,
    pub telegraph_timer: f32,
    pub attack_timer: f32,
    pub phase_flash: f32,
    pub attacks_landed: u32,
}

impl BossState {
    pub fn new(id: &str) -> Self {
        let definition = BossDefinition::for_id(id);
        Self {
            health: definition.max_health,
            definition,
            phase: BossPhase::Approach,
            telegraph_timer: definition.telegraph_seconds,
            attack_timer: 3.2,
            phase_flash: 0.0,
            attacks_landed: 0,
        }
    }

    pub fn health_ratio(&self) -> f32 {
        (self.health / self.definition.max_health).clamp(0.0, 1.0)
    }

    pub fn is_defeated(&self) -> bool {
        self.phase == BossPhase::Defeated
    }

    pub fn damage(&mut self, amount: f32) -> Option<BossEvent> {
        if self.is_defeated() {
            return None;
        }
        self.health = (self.health - amount.max(0.0)).max(0.0);
        if self.health <= 0.0 {
            self.phase = BossPhase::Defeated;
            return Some(BossEvent::Victory);
        }
        let ratio = self.health_ratio();
        let next = if ratio <= self.definition.enraged_at {
            BossPhase::Enraged
        } else if ratio <= self.definition.phase_two_at {
            BossPhase::Second
        } else {
            BossPhase::First
        };
        if next != self.phase {
            self.phase = next;
            self.phase_flash = 0.5;
            Some(BossEvent::PhaseChanged(next))
        } else {
            None
        }
    }

    pub fn update(&mut self, dt: f32, target: Vec2) -> Vec<BossEvent> {
        if self.is_defeated() {
            return Vec::new();
        }
        self.phase_flash = (self.phase_flash - dt).max(0.0);
        let mut events = Vec::new();
        if self.phase == BossPhase::Approach {
            self.telegraph_timer -= dt;
            if self.telegraph_timer <= 0.0 {
                self.phase = BossPhase::First;
                events.push(BossEvent::PhaseChanged(self.phase));
            } else {
                events.push(BossEvent::Telegraph {
                    phase: self.phase,
                    target,
                });
                return events;
            }
        }
        self.attack_timer -= dt;
        if self.attack_timer <= 0.0 {
            let cadence = match self.phase {
                BossPhase::First => 3.0,
                BossPhase::Second => 2.15,
                BossPhase::Enraged => 1.45,
                _ => 3.0,
            };
            self.attack_timer = cadence;
            self.attacks_landed += 1;
            events.push(BossEvent::Attack {
                damage: self.definition.attack_damage
                    * if self.phase == BossPhase::Enraged {
                        1.35
                    } else {
                        1.0
                    },
                target,
            });
        } else if self.attack_timer <= 0.8 {
            events.push(BossEvent::Telegraph {
                phase: self.phase,
                target,
            });
        }
        events
    }
}

#[cfg(test)]
mod tests;
