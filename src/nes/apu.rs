use super::nes::{State, AUDIO_SAMPLES_PER_FRAME};

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const DUTY_TABLE: [[bool; 8]; 4] = [
    [false, false, false, false, false, false, false, true],
    [false, false, false, false, false, false, true, true],
    [false, false, false, false, true, true, true, true],
    [true, true, true, true, true, true, false, false],
];

pub struct ApuState {
    /// Audio buffer (one frame's worth).
    pub audio_buffer: [f32; AUDIO_SAMPLES_PER_FRAME],
    /// Number of CPU cycles in this frame.
    frame_cycle_counter: usize,

    // Last CPU cycle that we emulated at.
    last_cpu_cycle: u64,
    cpu_cycles: u64,

    // Pulse 1
    pulse1_enabled: bool,
    pulse1_duty_table: &'static [bool; 8],
    pulse1_duty_counter: usize,
    pulse1_volume: u8,
    pulse1_freq_counter: u16,
    pulse1_freq_timer: u16,
    pulse1_length_counter: u8,
    pulse1_length_enabled: bool,
}

impl ApuState {
    pub fn new() -> ApuState {
        ApuState {
            audio_buffer: [0.0f32; AUDIO_SAMPLES_PER_FRAME],
            frame_cycle_counter: 0,

            last_cpu_cycle: 0,
            cpu_cycles: 0,

            pulse1_enabled: false,
            pulse1_duty_table: &DUTY_TABLE[0],
            pulse1_duty_counter: 0,
            pulse1_volume: 0,
            pulse1_freq_counter: 0,
            pulse1_freq_timer: 0,
            pulse1_length_counter: 0,
            pulse1_length_enabled: false,
        }
    }
}

pub fn complete_frame(s: &mut State) {
    catch_up(s);
    // Divide by average.
    for i in s.apu.audio_buffer.iter_mut() {
        *i *= 1.0 / 38.0;
    }
}

pub fn start_frame(s: &mut State) {
    s.apu.frame_cycle_counter = 0;

    for i in s.apu.audio_buffer.iter_mut() {
        *i = 0.0;
    }
}

pub fn catch_up(s: &mut State) {
    let cpu_cycles = s.cpu.cycles - s.apu.last_cpu_cycle;
    emulate(s, cpu_cycles);
}

pub fn emulate(s: &mut State, cycles: u64) {
    s.apu.last_cpu_cycle = s.cpu.cycles;

    for _ in 0..cycles {
        // APU cycles are every other CPU cycle.
        s.apu.frame_cycle_counter += 1;
        s.apu.cpu_cycles += 1;
        if s.apu.cpu_cycles & 0x1 != 1 {
            continue;
        }

        // Clock Pulse 1
        if s.apu.pulse1_freq_counter > 0 {
            s.apu.pulse1_freq_counter -= 1;
        } else {
            s.apu.pulse1_freq_counter = s.apu.pulse1_freq_timer;
            s.apu.pulse1_duty_counter = (s.apu.pulse1_duty_counter + 1) & 0x7;
        }

        // Compute output.
        let pulse1_out = if s.apu.pulse1_duty_table[s.apu.pulse1_duty_counter]
            && s.apu.pulse1_length_counter != 0
        {
            // TODO decay
            s.apu.pulse1_volume
        } else {
            0
        };

        // ~38 per thing
        let sample = (pulse1_out as f32) / (30.0f32);
        let index = (s.apu.frame_cycle_counter / 38) as usize;
        s.apu.audio_buffer[index] += sample;
    }
}

pub fn peek_register(s: &mut State, _register: u16) -> u8 {
    catch_up(s);
    0
}

pub fn poke_register(s: &mut State, register: u16, data: u8) {
    catch_up(s);

    match register {
        0x4000 => {
            s.apu.pulse1_duty_table = &DUTY_TABLE[((data & 0b1100_0000) >> 6) as usize];
            s.apu.pulse1_volume = data & 0b0000_1111;
            s.apu.pulse1_length_enabled = (data & 0b0010_0000) != 0;
            // TODO rest
        }
        0x4001 => {
            // TODO sweep
        }
        0x4002 => {
            s.apu.pulse1_freq_timer &= 0xFF00;
            s.apu.pulse1_freq_timer |= data as u16;
        }
        0x4003 => {
            s.apu.pulse1_freq_timer &= 0x00FF;
            s.apu.pulse1_freq_timer |= ((data as u16) & 0b111) << 8;

            if s.apu.pulse1_enabled {
                s.apu.pulse1_length_counter = LENGTH_TABLE[(data >> 3) as usize];
            }

            s.apu.pulse1_freq_counter = s.apu.pulse1_freq_timer;
            s.apu.pulse1_duty_counter = 0;
            //  decay_reset_flag =  true
        }
        0x4015 => {
            s.apu.pulse1_enabled = (data & 0b0000_0001) > 0;
            if !s.apu.pulse1_enabled {
                s.apu.pulse1_length_counter = 0;
            }

            // TODO: other channels
        }
        _ => {}
    }
}
