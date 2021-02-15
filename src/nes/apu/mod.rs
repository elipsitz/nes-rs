use super::nes::{State, AUDIO_SAMPLES_PER_FRAME};

mod pulse;

const FRAME_INTERVAL: u64 = 7457;

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

    pulse1: pulse::Pulse,
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

            pulse1: pulse::Pulse::new(),
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

        s.apu.pulse1.clock();

        // Compute output.
        let pulse1_out = s.apu.pulse1.output();

        // ~38 per thing
        let sample = (pulse1_out as f32) / (30.0f32);
        let index = (s.apu.frame_cycle_counter / 38) as usize;
        s.apu.audio_buffer[index] += sample;
    }
}

fn handle_frame_quarter(s: &mut State) {
    s.apu.pulse1.clock_frame_quarter();
}

fn handle_frame_half(s: &mut State) {
    s.apu.pulse1.clock_frame_half();
}

pub fn peek_register(s: &mut State, _register: u16) -> u8 {
    catch_up(s);
    0
}

pub fn poke_register(s: &mut State, register: u16, data: u8) {
    catch_up(s);

    match register {
        0x4000..=0x4003 => {
            s.apu.pulse1.poke_register(register, data);
        }
        0x4015 => {
            s.apu.pulse1.set_enable_flag((data & 0b0000_0001) != 0);

            // TODO: other channels
        }
        0x4017 => {
            // TODO: frame counter + interrupt
            // println!("--- {:b}", data);
        }
        _ => {}
    }
}
