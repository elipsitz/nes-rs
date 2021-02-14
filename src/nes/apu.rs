use super::nes::AUDIO_SAMPLES_PER_FRAME;

pub struct ApuState {
    /// Audio buffer (one frame's worth).
    pub audio_buffer: [f32; AUDIO_SAMPLES_PER_FRAME],
}

impl ApuState {
    pub fn new() -> ApuState {
        // Dummy sine wave to test.
        let mut buffer = [0.0f32; AUDIO_SAMPLES_PER_FRAME];
        for i in 0..AUDIO_SAMPLES_PER_FRAME {
            let period = 100f32;
            let phase = (i as f32) * (3.14f32 / (period as f32));
            buffer[i] = phase.sin();
        }

        ApuState {
            audio_buffer: buffer,
        }
    }
}
