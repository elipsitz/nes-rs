use serde::{Deserialize, Serialize};

const PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

#[derive(Serialize, Deserialize)]
pub struct Noise {
    enabled: bool,
    noise_shift: u16,
    freq_timer: u16,
    freq_counter: u16,
    shift_mode: u8,
    length_counter: u8,
    length_enabled: bool,
    volume: u8,
    decay_enabled: bool,
    decay_reset_flag: bool,
    decay_hidden_volume: u8,
    decay_counter: u8,
    decay_loop: bool,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            enabled: false,
            noise_shift: 1,
            freq_timer: 0,
            freq_counter: 0,
            shift_mode: 0,
            length_counter: 0,
            length_enabled: false,
            volume: 0,
            decay_enabled: false,
            decay_reset_flag: false,
            decay_hidden_volume: 0,
            decay_counter: 0,
            decay_loop: false,
        }
    }

    /// Clocked every APU cycle.
    pub fn clock(&mut self) {
        if self.freq_counter > 0 {
            self.freq_counter -= 1;
        } else {
            self.freq_counter = self.freq_timer;

            // LFSR
            let bit = if self.shift_mode == 0 {
                (self.noise_shift ^ (self.noise_shift >> 1)) & 0b1
            } else {
                (self.noise_shift ^ (self.noise_shift >> 6)) & 0b1
            };
            self.noise_shift |= bit << 15;
            self.noise_shift >>= 1;
        }
    }

    pub fn clock_frame_quarter(&mut self) {
        // Envelope
        if self.decay_reset_flag {
            self.decay_reset_flag = false;
            self.decay_hidden_volume = 0xF;
            self.decay_counter = self.volume;
        } else {
            if self.decay_counter > 0 {
                self.decay_counter -= 1;
            } else {
                self.decay_counter = self.volume;
                if self.decay_hidden_volume > 0 {
                    self.decay_hidden_volume -= 1;
                } else if self.decay_loop {
                    self.decay_hidden_volume = 0xF;
                }
            }
        }
    }

    pub fn clock_frame_half(&mut self) {
        // Clock Length.
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if (self.noise_shift & 0b1) == 0 && self.length_counter != 0 {
            if self.decay_enabled {
                self.decay_hidden_volume
            } else {
                self.volume
            }
        } else {
            0
        }
    }

    pub fn poke_register(&mut self, register: u16, data: u8) {
        match register {
            0x400C => {
                self.volume = data & 0b0000_1111;
                self.length_enabled = (data & 0b0010_0000) == 0;
                self.decay_enabled = (data & 0b0001_0000) == 0;
                self.decay_loop = (data & 0b0010_0000) != 0;
            }
            0x400D => {}
            0x400E => {
                self.freq_timer = PERIOD_TABLE[(data & 0b1111) as usize];
                self.shift_mode = data & 0b1000_0000;
            }
            0x400F => {
                if self.enabled {
                    self.length_counter = super::LENGTH_TABLE[(data >> 3) as usize];
                }

                self.decay_reset_flag = true;
            }
            _ => unreachable!(),
        }
    }

    pub fn set_enable_flag(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !self.enabled {
            self.length_counter = 0;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.length_counter > 0
    }
}
