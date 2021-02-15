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

pub struct Pulse {
    enabled: bool,
    duty_table: &'static [bool; 8],
    duty_counter: usize,
    volume: u8,
    freq_counter: u16,
    freq_timer: u16,
    length_counter: u8,
    length_enabled: bool,
    decay_enabled: bool,
    decay_reset_flag: bool,
    decay_hidden_volume: u8,
    decay_counter: u8,
    decay_loop: bool,
    sweep_timer: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_reload: bool,
    sweep_enabled: bool,
    sweep_counter: u8,

    // 1 for Pulse 1, 0 for Pulse 2.
    sweep_negate_constant: u16,
}

impl Pulse {
    pub fn new_pulse1() -> Pulse {
        Pulse::new(1)
    }

    pub fn new_pulse2() -> Pulse {
        Pulse::new(0)
    }

    fn new(sweep_negate_constant: u16) -> Pulse {
        Pulse {
            enabled: false,
            duty_table: &DUTY_TABLE[0],
            duty_counter: 0,
            volume: 0,
            freq_counter: 0,
            freq_timer: 0,
            length_counter: 0,
            length_enabled: false,
            decay_enabled: false,
            decay_reset_flag: false,
            decay_hidden_volume: 0,
            decay_counter: 0,
            decay_loop: false,
            sweep_timer: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_reload: false,
            sweep_enabled: false,
            sweep_counter: 0,
            sweep_negate_constant,
        }
    }

    /// Clocked every APU cycle (every 2 CPU cycles).
    pub fn clock(&mut self) {
        if self.freq_counter > 0 {
            self.freq_counter -= 1;
        } else {
            self.freq_counter = self.freq_timer;
            self.duty_counter = (self.duty_counter + 1) & 0x7;
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
        // Clock Sweep.
        if self.sweep_reload {
            self.sweep_counter = self.sweep_timer;
            self.sweep_reload = false;
        } else if self.sweep_counter > 0 {
            self.sweep_counter -= 1;
        } else {
            self.sweep_counter = self.sweep_timer;

            if self.sweep_enabled && !self.is_sweep_silencing() {
                if self.sweep_negate {
                    self.freq_timer -= self.freq_timer >> self.sweep_shift;
                    self.freq_timer -= self.sweep_negate_constant;
                } else {
                    self.freq_timer += self.freq_timer >> self.sweep_shift;
                }
            }
        }

        // Clock Length.
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn output(&self) -> u8 {
        if self.duty_table[self.duty_counter]
            && self.length_counter != 0
            && !self.is_sweep_silencing()
        {
            if self.decay_enabled {
                self.decay_hidden_volume
            } else {
                self.volume
            }
        } else {
            0
        }
    }

    fn is_sweep_silencing(&self) -> bool {
        if self.freq_timer < 8 {
            true
        } else if !self.sweep_negate
            && (self.freq_timer + (self.freq_timer >> self.sweep_shift)) >= 0x800
        {
            true
        } else {
            false
        }
    }

    pub fn poke_register(&mut self, register: u16, data: u8) {
        match register & 0b11 {
            0 => {
                self.duty_table = &DUTY_TABLE[((data & 0b1100_0000) >> 6) as usize];
                self.volume = data & 0b0000_1111;
                self.length_enabled = (data & 0b0010_0000) == 0;
                self.decay_enabled = (data & 0b0001_0000) == 0;
                self.decay_loop = (data & 0b0010_0000) != 0;
            }
            1 => {
                self.sweep_timer = (data & 0b0111_0000) >> 4;
                self.sweep_negate = (data & 0b0000_1000) != 0;
                self.sweep_shift = data & 0b0000_0111;
                self.sweep_reload = true;
                self.sweep_enabled = ((data & 0b1000_0000) != 0) && (self.sweep_shift != 0);
            }
            2 => {
                self.freq_timer &= 0xFF00;
                self.freq_timer |= data as u16;
            }
            3 => {
                self.freq_timer &= 0x00FF;
                self.freq_timer |= ((data as u16) & 0b111) << 8;

                if self.enabled {
                    self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
                }

                self.freq_counter = self.freq_timer;
                self.duty_counter = 0;
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
}
