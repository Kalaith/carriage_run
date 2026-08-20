//! Transient mission feedback owned separately from simulation state.

use macroquad_toolkit::fx::{FloatingTextLayer, ParticleSystem, ScreenShake};

/// Short-lived feedback that can be reset or updated without changing the
/// authoritative route outcome. Keeping it together prevents the mission
/// model from growing a separate top-level field for every visual effect.
#[derive(Debug, Clone)]
pub struct MissionEffects {
    pub float_texts: FloatingTextLayer,
    pub particles: ParticleSystem,
    pub screen_shake: ScreenShake,
    pub hit_stop: f32,
}

impl MissionEffects {
    pub fn new(float_text_life: f32) -> Self {
        let mut float_texts = FloatingTextLayer::new();
        float_texts.default_lifetime = float_text_life;
        float_texts.default_rise_speed = 26.0;
        float_texts.shadow = false;

        Self {
            float_texts,
            particles: ParticleSystem::new(),
            screen_shake: ScreenShake::new(8.0),
            hit_stop: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.screen_shake.update(dt);
        self.float_texts.update(dt);
        self.particles.update(dt);
    }
}
