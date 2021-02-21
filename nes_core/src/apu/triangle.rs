use serde::{Deserialize, Serialize};

const SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

#[derive(Serialize, Deserialize)]
pub struct Triangle {
    enabled: bool,
    length_counter: u8,
    length_enabled: bool,
    freq_timer: u16,
    freq_counter: u16,
    sequence_counter: usize,
    linear_control: bool,
    linear_reload: bool,
    linear_load: u8,
    linear_counter: u8,
}

impl Triangle {
    pub fn new() -> Triangle {
        Triangle {
            enabled: false,
            length_counter: 0,
            length_enabled: false,
            freq_timer: 0,
            freq_counter: 0,
            sequence_counter: 0,
            linear_control: false,
            linear_reload: false,
            linear_load: 0,
            linear_counter: 0,
        }
    }

    /// Clocked every CPU cycle.
    pub fn clock(&mut self) {
        let ultrasonic = self.freq_timer < 2 && self.freq_counter == 0;
        let active = self.length_counter != 0 && self.linear_counter != 0 && !ultrasonic;

        if active {
            if self.freq_counter > 0 {
                self.freq_counter -= 1;
            } else {
                self.freq_counter = self.freq_timer;
                self.sequence_counter = (self.sequence_counter + 1) & 0x1F;
            }
        }
    }

    pub fn clock_frame_quarter(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.linear_load;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.linear_control {
            self.linear_reload = false;
        }
    }

    pub fn clock_frame_half(&mut self) {
        // Clock Length.
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        let ultrasonic = self.freq_timer < 2 && self.freq_counter == 0;
        if ultrasonic {
            7
        } else {
            SEQUENCE[self.sequence_counter]
        }
    }

    pub fn poke_register(&mut self, register: u16, data: u8) {
        match register {
            0x4008 => {
                self.linear_control = (data & 0b1000_0000) != 0;
                self.length_enabled = (data & 0b1000_0000) == 0;
                self.linear_load = data & 0b0111_1111;
            }
            0x4009 => {}
            0x400A => {
                self.freq_timer &= 0xFF00;
                self.freq_timer |= data as u16;
            }
            0x400B => {
                self.freq_timer &= 0x00FF;
                self.freq_timer |= ((data as u16) & 0b111) << 8;

                if self.enabled {
                    self.length_counter = super::LENGTH_TABLE[(data >> 3) as usize];
                }

                self.linear_reload = true;
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
