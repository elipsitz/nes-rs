use super::nes::{State, AUDIO_SAMPLES_PER_FRAME};

pub struct ApuState {
    /// Audio buffer (one frame's worth).
    pub audio_buffer: [f32; AUDIO_SAMPLES_PER_FRAME],

    // Last CPU cycle that we emulated at.
    last_cpu_cycle: u64,
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
            last_cpu_cycle: 0,
        }
    }
}

pub fn catch_up(s: &mut State) {
    let cpu_cycles = s.cpu.cycles - s.apu.last_cpu_cycle;
    emulate(s, cpu_cycles * 2);
}

pub fn emulate(s: &mut State, _cycles: u64) {
    s.apu.last_cpu_cycle = s.cpu.cycles;
}

pub fn peek_register(s: &mut State, _register: u16) -> u8 {
    catch_up(s);
    0
}

pub fn poke_register(s: &mut State, _register: u16, _data: u8) {
    catch_up(s);
}
