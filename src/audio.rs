//! Event-driven audio boundary. Sound files are optional so a missing release
//! audio pack never prevents the simulation from booting.

use crate::settings::RuntimeSettings;
use crate::state::Screen;
use macroquad::audio::PlaySoundParams;
use macroquad_toolkit::audio::SoundManager;
use macroquad_toolkit::synth::{render_wav, SynthConfig, Voice, Wave};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioCue {
    UiConfirm,
    UiCancel,
    UiHover,
    Hit,
    CarriageImpact,
    Hazard,
    BossTelegraph,
    Victory,
    Defeat,
    Music,
}

pub struct GameAudio {
    pub manager: SoundManager<AudioCue>,
    pub screen_music: Option<Screen>,
    page_focused: bool,
    music_started: bool,
}

impl GameAudio {
    pub fn new() -> Self {
        Self {
            manager: SoundManager::new(),
            screen_music: None,
            page_focused: true,
            music_started: false,
        }
    }

    pub async fn load_generated(&mut self) {
        let config = SynthConfig {
            sample_rate: 22_050,
            master_gain: 0.20,
        };
        for cue in AudioCue::all_effects() {
            let bytes = render_wav(&voices_for(cue), &config, cue as u64 + 0x00CA_771E);
            let _ = self.manager.load_sound_bytes(cue, &bytes).await;
        }
        let music = render_wav(
            &[
                Voice::tone(0.0, 1.4, 196.0, 0.14).wave(Wave::Triangle),
                Voice::tone(0.0, 1.4, 247.0, 0.10).wave(Wave::Triangle),
                Voice::tone(1.5, 1.4, 220.0, 0.14).wave(Wave::Triangle),
                Voice::tone(1.5, 1.4, 277.0, 0.10).wave(Wave::Triangle),
            ],
            &SynthConfig {
                sample_rate: 22_050,
                master_gain: 0.10,
            },
            0xCA77_1A6E,
        );
        let _ = self.manager.load_sound_bytes(AudioCue::Music, &music).await;
    }

    pub fn set_screen(&mut self, screen: Screen) {
        if self.screen_music != Some(screen) {
            self.screen_music = Some(screen);
            self.music_started = false;
        }
        if !self.music_started
            && matches!(
                screen,
                Screen::Title | Screen::MissionMap | Screen::Playing | Screen::Journey
            )
        {
            self.manager.play_raw(
                AudioCue::Music,
                PlaySoundParams {
                    looped: true,
                    volume: 0.26,
                },
            );
            self.music_started = true;
        }
    }

    pub fn set_page_focused(&mut self, focused: bool, settings: &RuntimeSettings) {
        self.page_focused = focused;
        self.manager.visible = settings.audio_visible(focused);
    }

    pub fn apply_settings(&mut self, settings: &RuntimeSettings) {
        self.manager.sfx_volume = settings.display.effective_sfx_volume();
        self.manager.music_volume = settings.display.effective_music_volume();
        self.manager.visible = settings.audio_visible(self.page_focused);
    }

    pub fn ui(&self, cue: AudioCue, settings: &RuntimeSettings) {
        self.manager
            .play_sfx(cue, settings.display.effective_sfx_volume());
    }

    pub fn combat(&self, cue: AudioCue, intensity: f32, settings: &RuntimeSettings) {
        self.manager.play_sfx(
            cue,
            intensity.clamp(0.0, 1.0) * settings.display.effective_sfx_volume(),
        );
    }
}

impl AudioCue {
    fn all_effects() -> [Self; 9] {
        [
            Self::UiConfirm,
            Self::UiCancel,
            Self::UiHover,
            Self::Hit,
            Self::CarriageImpact,
            Self::Hazard,
            Self::BossTelegraph,
            Self::Victory,
            Self::Defeat,
        ]
    }
}

fn voices_for(cue: AudioCue) -> Vec<Voice> {
    match cue {
        AudioCue::UiConfirm => vec![Voice::tone(0.0, 0.10, 760.0, 0.45)],
        AudioCue::UiCancel => vec![Voice::tone(0.0, 0.12, 320.0, 0.40)],
        AudioCue::UiHover => vec![Voice::tone(0.0, 0.06, 1_180.0, 0.20)],
        AudioCue::Hit => vec![Voice::tone(0.0, 0.08, 220.0, 0.35).wave(Wave::Noise)],
        AudioCue::CarriageImpact => {
            vec![Voice::tone(0.0, 0.18, 110.0, 0.50).wave(Wave::Noise)]
        }
        AudioCue::Hazard => vec![Voice::tone(0.0, 0.22, 180.0, 0.35).glide(80.0)],
        AudioCue::BossTelegraph => vec![Voice::tone(0.0, 0.32, 160.0, 0.42).glide(420.0)],
        AudioCue::Victory => vec![
            Voice::tone(0.0, 0.20, 523.0, 0.35),
            Voice::tone(0.16, 0.24, 659.0, 0.35),
            Voice::tone(0.34, 0.36, 784.0, 0.38),
        ],
        AudioCue::Defeat => vec![Voice::tone(0.0, 0.38, 260.0, 0.40).glide(100.0)],
        AudioCue::Music => Vec::new(),
    }
}

impl Default for GameAudio {
    fn default() -> Self {
        Self::new()
    }
}
