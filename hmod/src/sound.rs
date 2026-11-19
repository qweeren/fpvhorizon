//! The drone's sounds: the motors' whine following their speed, and the elytra's wind rushing
//! louder the faster it flies. A playing sound keeps the pitch it started with, so the motors
//! are one recording looped at pitches a semitone apart, crossfaded by motor speed: neighbours
//! differ by 6%, too little to hear as a step. Only the bands next to the motors' pitch play.

use crate::drone::DroneState;
use mod_api::{ClientHost, SoundPlay, SoundRef, SoundSource};

/// Two octaves, a semitone a band: pitch 0.5 (band 0) to 2.0 (band 24).
const BANDS: usize = 25;
const MOTOR: &str = "fpvdrone:motor";
const WIND: &str = "minecraft:item.elytra.flying";
/// A fade to 0 stops a sound for good: quiet loops sit just above it.
const SILENT: f32 = 0.001;

fn band_pitch(band: usize) -> f32 {
    0.5 * 2f32.powf(band as f32 / 12.0)
}

pub struct Audio {
    motors: [Option<SoundRef>; BANDS],
    wind: Option<SoundRef>,
}

impl Audio {
    pub const NONE: Self = Self { motors: [None; BANDS], wind: None };

    fn play(host: &dyn ClientHost, id: &str, pitch: f32) -> Option<SoundRef> {
        let mut play = SoundPlay::ui(id).with_pitch(pitch).with_volume(SILENT).looping(0);
        play.source = SoundSource::Players;
        host.play_local_sound(&play, 0)
    }

    /// Steers the sounds for `state` (None when there is no drone out), `far` being how much
    /// distance quietens it (1 when the camera is on the drone), with the settings' volumes.
    pub fn update(&mut self, host: &dyn ClientHost, state: Option<&DroneState>, far: f32, (motor, wind): (f32, f32)) {
        let Some(state) = state else {
            self.stop(host);
            return;
        };

        // Motor speed picks the pitch on a log scale (hover near the recording's own pitch).
        let rpm = state.physics.rpm_fraction();
        let band = rpm * (BANDS - 1) as f32;
        let loudness = if rpm < 0.01 { 0.0 } else { (0.15 + 0.85 * rpm) * motor * far };
        for (i, slot) in self.motors.iter_mut().enumerate() {
            let distance = (band - i as f32).abs();
            if distance < 1.5 {
                if slot.is_none() {
                    *slot = Self::play(host, MOTOR, band_pitch(i));
                }
                let weight = (1.0 - distance).max(0.0);
                if let Some(sound) = slot {
                    sound.fade(host, (weight * loudness).max(SILENT), 1);
                }
            } else if distance > 2.5
                && let Some(sound) = slot.take()
            {
                sound.stop(host);
            }
        }

        if self.wind.is_none() {
            self.wind = Self::play(host, WIND, 1.0);
        }
        let speed = state.velocity.length();
        let rush = ((speed - 4.0) / 25.0).clamp(0.0, 1.0) * wind * far;
        if let Some(sound) = &self.wind {
            sound.fade(host, rush.max(SILENT), 1);
        }
    }

    pub fn stop(&mut self, host: &dyn ClientHost) {
        for sound in self.motors.iter_mut().filter_map(Option::take) {
            sound.stop(host);
        }
        if let Some(sound) = self.wind.take() {
            sound.stop(host);
        }
    }
}
