use super::nes::{State, AUDIO_SAMPLES_PER_FRAME};

const FRAME_INTERVAL: u64 = 7457;

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

    // Frame Counter
    sequence_counter: u64,
    next_seq_phase: usize,

    // Pulse 1
    pulse1_enabled: bool,
    pulse1_duty_table: &'static [bool; 8],
    pulse1_duty_counter: usize,
    pulse1_volume: u8,
    pulse1_freq_counter: u16,
    pulse1_freq_timer: u16,
    pulse1_length_counter: u8,
    pulse1_length_enabled: bool,
    pulse1_decay_enabled: bool,
    pulse1_decay_reset_flag: bool,
    pulse1_decay_hidden_volume: u8,
    pulse1_decay_counter: u8,
    pulse1_decay_loop: bool,
    pulse1_sweep_timer: u8,
    pulse1_sweep_negate: bool,
    pulse1_sweep_shift: u8,
    pulse1_sweep_reload: bool,
    pulse1_sweep_enabled: bool,
    pulse1_sweep_counter: u8,
}

impl ApuState {
    pub fn new() -> ApuState {
        ApuState {
            audio_buffer: [0.0f32; AUDIO_SAMPLES_PER_FRAME],
            frame_cycle_counter: 0,

            last_cpu_cycle: 0,
            cpu_cycles: 0,

            sequence_counter: FRAME_INTERVAL,
            next_seq_phase: 0,

            pulse1_enabled: false,
            pulse1_duty_table: &DUTY_TABLE[0],
            pulse1_duty_counter: 0,
            pulse1_volume: 0,
            pulse1_freq_counter: 0,
            pulse1_freq_timer: 0,
            pulse1_length_counter: 0,
            pulse1_length_enabled: false,
            pulse1_decay_enabled: false,
            pulse1_decay_reset_flag: false,
            pulse1_decay_hidden_volume: 0,
            pulse1_decay_counter: 0,
            pulse1_decay_loop: false,
            pulse1_sweep_timer: 0,
            pulse1_sweep_negate: false,
            pulse1_sweep_shift: 0,
            pulse1_sweep_reload: false,
            pulse1_sweep_enabled: false,
            pulse1_sweep_counter: 0,
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
        // Frame Counter (clocked on CPU).
        s.apu.sequence_counter -= 1;
        if s.apu.sequence_counter == 0 {
            // 4 cycle only
            match s.apu.next_seq_phase {
                0 => {
                    handle_frame_quarter(s);
                }
                1 => {
                    handle_frame_quarter(s);
                    handle_frame_half(s);
                }
                2 => {
                    handle_frame_quarter(s);
                }
                3 => {
                    handle_frame_quarter(s);
                    handle_frame_half(s);
                }
                _ => unreachable!(),
            }
            // TODO: interrupt?

            // TODO handle 5 cycle
            s.apu.next_seq_phase = (s.apu.next_seq_phase + 1) % 4;
            s.apu.sequence_counter = 7457;
        }

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
            && !pulse1_sweep_silence(s)
        {
            if s.apu.pulse1_decay_enabled {
                s.apu.pulse1_decay_hidden_volume
            } else {
                s.apu.pulse1_volume
            }
        } else {
            0
        };

        // ~38 per thing
        let sample = (pulse1_out as f32) / (30.0f32);
        let index = (s.apu.frame_cycle_counter / 38) as usize;
        s.apu.audio_buffer[index] += sample;
    }
}

fn handle_frame_quarter(s: &mut State) {
    // Pulse 1 Envelope
    if s.apu.pulse1_decay_reset_flag {
        s.apu.pulse1_decay_reset_flag = false;
        s.apu.pulse1_decay_hidden_volume = 0xF;
        s.apu.pulse1_decay_counter = s.apu.pulse1_volume;
    } else {
        if s.apu.pulse1_decay_counter > 0 {
            s.apu.pulse1_decay_counter -= 1;
        } else {
            s.apu.pulse1_decay_counter = s.apu.pulse1_volume;
            if s.apu.pulse1_decay_hidden_volume > 0 {
                s.apu.pulse1_decay_hidden_volume -= 1;
            } else if s.apu.pulse1_decay_loop {
                s.apu.pulse1_decay_hidden_volume = 0xF;
            }
        }
    }
}

fn handle_frame_half(s: &mut State) {
    // Pulse 1 Clock Sweep.
    if s.apu.pulse1_sweep_reload {
        s.apu.pulse1_sweep_counter = s.apu.pulse1_sweep_timer;
        s.apu.pulse1_sweep_reload = false;
    } else if s.apu.pulse1_sweep_counter > 0 {
        s.apu.pulse1_sweep_counter -= 1;
    } else {
        s.apu.pulse1_sweep_counter = s.apu.pulse1_sweep_timer;

        if s.apu.pulse1_sweep_enabled && !pulse1_sweep_silence(s) {
            if s.apu.pulse1_sweep_negate {
                s.apu.pulse1_freq_timer -= s.apu.pulse1_freq_timer >> s.apu.pulse1_sweep_shift;
                s.apu.pulse1_freq_timer -= 1;
                // note: pulse2 has no additional -1
            } else {
                s.apu.pulse1_freq_timer += s.apu.pulse1_freq_timer >> s.apu.pulse1_sweep_shift;
            }
        }
    }

    // Pulse 1 Clock Length.
    if s.apu.pulse1_length_enabled && s.apu.pulse1_length_counter > 0 {
        s.apu.pulse1_length_counter -= 1;
    }
}

fn pulse1_sweep_silence(s: &State) -> bool {
    if s.apu.pulse1_freq_timer < 8 {
        true
    } else if !s.apu.pulse1_sweep_negate
        && (s.apu.pulse1_freq_timer + (s.apu.pulse1_freq_timer >> s.apu.pulse1_sweep_shift))
            >= 0x800
    {
        true
    } else {
        false
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
            s.apu.pulse1_length_enabled = (data & 0b0010_0000) == 0;
            s.apu.pulse1_decay_enabled = (data & 0b0001_0000) == 0;
            s.apu.pulse1_decay_loop = (data & 0b0010_0000) != 0;
        }
        0x4001 => {
            s.apu.pulse1_sweep_timer = (data & 0b0111_0000) >> 4;
            s.apu.pulse1_sweep_negate = (data & 0b0000_1000) != 0;
            s.apu.pulse1_sweep_shift = data & 0b0000_0111;
            s.apu.pulse1_sweep_reload = true;
            s.apu.pulse1_sweep_enabled =
                ((data & 0b1000_0000) != 0) && (s.apu.pulse1_sweep_shift != 0);
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
            s.apu.pulse1_decay_reset_flag = true;
        }
        0x4015 => {
            s.apu.pulse1_enabled = (data & 0b0000_0001) > 0;
            if !s.apu.pulse1_enabled {
                s.apu.pulse1_length_counter = 0;
            }

            // TODO: other channels
        }
        0x4017 => {
            // TODO: frame counter + interrupt
            println!("--- {:b}", data);
        }
        _ => {}
    }
}
