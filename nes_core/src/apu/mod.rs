use super::nes::{State, AUDIO_SAMPLES_PER_FRAME};
use serde::{Deserialize, Serialize};
use serde_big_array::big_array;

mod dmc;
mod noise;
mod pulse;
mod triangle;

const FRAME_INTERVAL: u64 = 7457;
const FULL_AUDIO_BUFFER_LEN: usize = AUDIO_SAMPLES_PER_FRAME * 40;

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

big_array! { BigArray; AUDIO_SAMPLES_PER_FRAME, FULL_AUDIO_BUFFER_LEN }

#[derive(Serialize, Deserialize)]
pub struct ApuState {
    /// Downsampled audio buffer (one frame's worth).
    // #[serde(skip)]
    #[serde(with = "BigArray")]
    pub audio_buffer: [f32; AUDIO_SAMPLES_PER_FRAME],
    /// Non-downsampled audio buffer.
    //#[serde(skip)]
    #[serde(with = "BigArray")]
    full_audio_buffer: [f32; FULL_AUDIO_BUFFER_LEN],
    audio_index: usize,
    /// Number of CPU cycles in this frame.
    frame_cycle_counter: usize,

    // Last CPU cycle that we emulated at.
    last_cpu_cycle: u64,
    cpu_cycles: u64,

    // Frame Counter
    sequence_counter: u64,
    next_seq_phase: usize,
    sequencer_mode: u8,
    irq_enabled: bool,
    irq_pending: bool,

    // Units
    pulse1: pulse::Pulse,
    pulse2: pulse::Pulse,
    triangle: triangle::Triangle,
    noise: noise::Noise,
    dmc: dmc::Dmc,
}

impl ApuState {
    pub fn new() -> ApuState {
        ApuState {
            audio_buffer: [0.0f32; AUDIO_SAMPLES_PER_FRAME],
            full_audio_buffer: [0.0f32; FULL_AUDIO_BUFFER_LEN],
            audio_index: 0,
            frame_cycle_counter: 0,

            last_cpu_cycle: 0,
            cpu_cycles: 0,

            sequence_counter: FRAME_INTERVAL,
            next_seq_phase: 0,
            sequencer_mode: 0,
            irq_enabled: false,
            irq_pending: false,

            pulse1: pulse::Pulse::new_pulse1(),
            pulse2: pulse::Pulse::new_pulse2(),
            triangle: triangle::Triangle::new(),
            noise: noise::Noise::new(),
            dmc: dmc::Dmc::new(),
        }
    }
}

pub fn complete_frame(s: &mut State) {
    catch_up(s);

    // Downsample full buffer into audio_buffer (nearest neighbor).
    let num_samples = s.apu.audio_index as f32;
    for i in 0..AUDIO_SAMPLES_PER_FRAME {
        let sample_index = ((i as f32) / (AUDIO_SAMPLES_PER_FRAME as f32)) * num_samples;
        let sample = s.apu.full_audio_buffer[sample_index as usize];
        s.apu.audio_buffer[i] = sample;
    }
}

pub fn start_frame(s: &mut State) {
    s.apu.frame_cycle_counter = 0;
    s.apu.audio_index = 0;
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
            // 4 cycle.
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
                    if s.apu.sequencer_mode == 0 {
                        handle_frame_quarter(s);
                        handle_frame_half(s);
                        if s.apu.irq_enabled {
                            s.cpu.pending_interrupt = super::cpu::InterruptKind::IRQ;
                        }
                    }
                }
                4 => {
                    handle_frame_quarter(s);
                    handle_frame_half(s);
                }
                _ => unreachable!(),
            }

            s.apu.next_seq_phase =
                (s.apu.next_seq_phase + 1) % (4 + (s.apu.sequencer_mode as usize));
            s.apu.sequence_counter = 7457;
        }

        // Triangle gets clocked with the CPU.
        s.apu.triangle.clock();

        // APU cycles are every other CPU cycle.
        s.apu.frame_cycle_counter += 1;
        s.apu.cpu_cycles += 1;
        if s.apu.cpu_cycles & 0x1 != 1 {
            continue;
        }

        s.apu.pulse1.clock();
        s.apu.pulse2.clock();
        s.apu.noise.clock();
        dmc::Dmc::clock(s);

        // Compute subunit outputs.
        let pulse1_out = s.apu.pulse1.output() as f32;
        let pulse2_out = s.apu.pulse2.output() as f32;
        let triangle_out = s.apu.triangle.output() as f32;
        let noise_out = s.apu.noise.output() as f32;
        let dmc_out = s.apu.dmc.output() as f32;

        // Combine output. TODO make more efficient?
        let pulse_out = 95.88f32 / ((8128f32 / (pulse1_out + pulse2_out)) + 100f32);
        let tnd_denom = (triangle_out / 8227f32) + (noise_out / 12241f32) + (dmc_out / 22638f32);
        let tnd_out = 159.79f32 / ((1f32 / (tnd_denom)) + 100f32);
        let sample = pulse_out + tnd_out;

        // Write into the full audio buffer.
        s.apu.full_audio_buffer[s.apu.audio_index] = sample;
        s.apu.audio_index += 1;
    }
}

fn handle_frame_quarter(s: &mut State) {
    s.apu.pulse1.clock_frame_quarter();
    s.apu.pulse2.clock_frame_quarter();
    s.apu.triangle.clock_frame_quarter();
    s.apu.noise.clock_frame_quarter();
    s.apu.dmc.clock_frame_quarter();
}

fn handle_frame_half(s: &mut State) {
    s.apu.pulse1.clock_frame_half();
    s.apu.pulse2.clock_frame_half();
    s.apu.triangle.clock_frame_half();
    s.apu.noise.clock_frame_half();
    s.apu.dmc.clock_frame_half();
}

pub fn peek_register(s: &mut State, register: u16) -> u8 {
    catch_up(s);
    if register == 0x4015 {
        let val = (s.apu.pulse1.is_enabled() as u8)
            | ((s.apu.pulse2.is_enabled() as u8) << 1)
            | ((s.apu.triangle.is_enabled() as u8) << 2)
            | ((s.apu.noise.is_enabled() as u8) << 3)
            | ((s.apu.dmc.is_enabled() as u8) << 4)
            | ((s.apu.irq_pending as u8) << 6)
            | ((s.apu.dmc.is_irq_pending() as u8) << 7);
        s.apu.irq_pending = false;
        val
    } else {
        0
    }
}

pub fn poke_register(s: &mut State, register: u16, data: u8) {
    catch_up(s);

    match register {
        0x4000..=0x4003 => s.apu.pulse1.poke_register(register, data),
        0x4004..=0x4007 => s.apu.pulse2.poke_register(register, data),
        0x4008..=0x400B => s.apu.triangle.poke_register(register, data),
        0x400C..=0x400F => s.apu.noise.poke_register(register, data),
        0x4010..=0x4013 => s.apu.dmc.poke_register(register, data),
        0x4015 => {
            s.apu.pulse1.set_enable_flag((data & 0b0000_0001) != 0);
            s.apu.pulse2.set_enable_flag((data & 0b0000_0010) != 0);
            s.apu.triangle.set_enable_flag((data & 0b0000_0100) != 0);
            s.apu.noise.set_enable_flag((data & 0b0000_1000) != 0);
            s.apu.dmc.set_enable_flag((data & 0b0001_0000) != 0);
        }
        0x4017 => {
            s.apu.sequencer_mode = (data & 0b1000_0000) >> 7;
            s.apu.irq_enabled = (data & 0b0100_0000) == 0;
            s.apu.next_seq_phase = 0;
            s.apu.sequence_counter = FRAME_INTERVAL;

            if s.apu.sequence_counter == 1 {
                handle_frame_quarter(s);
                handle_frame_half(s);
            }
            if !s.apu.irq_enabled {
                s.apu.irq_pending = false;
            }
        }
        _ => {}
    }
}
