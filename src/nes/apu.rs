#![allow(dead_code)]

/// Samples per second.
pub const AUDIO_SAMPLE_RATE: usize = 48000;

/// Samples per frame.
pub const AUDIO_SAMPLES_PER_FRAME: usize = AUDIO_SAMPLE_RATE / 60;

pub struct ApuState {}

impl ApuState {
    pub fn new() -> ApuState {
        ApuState {}
    }
}
