pub struct Dmc {
    enabled: bool,
}

impl Dmc {
    pub fn new() -> Dmc {
        Dmc { enabled: false }
    }

    pub fn clock(&mut self) {}

    pub fn clock_frame_quarter(&mut self) {}

    pub fn clock_frame_half(&mut self) {}

    pub fn output(&self) -> u8 {
        0
    }

    pub fn poke_register(&mut self, register: u16, _data: u8) {
        match register {
            0x4010 => {}
            0x4011 => {}
            0x4012 => {}
            0x4013 => {}
            _ => unreachable!(),
        }
    }

    pub fn set_enable_flag(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        false
    }
}
