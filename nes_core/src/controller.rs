use crate::nes::State;

#[derive(Default, Debug)]
pub struct ControllerState {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    index: usize,
}

impl ControllerState {
    pub fn new() -> ControllerState {
        ControllerState::default()
    }

    pub fn read(&mut self) -> u8 {
        // TODO: simulate open bus
        // https://wiki.nesdev.com/w/index.php/Controller_reading#Unconnected_data_lines_and_open_bus

        let status = match self.index {
            0 => self.a,
            1 => self.b,
            2 => self.select,
            3 => self.start,
            4 => self.up,
            5 => self.down,
            6 => self.left,
            7 => self.right,
            _ => false,
        };
        self.index += 1;
        return status as u8;
    }
}

pub fn write(s: &mut State, data: u8) {
    let strobe = data & 0x1 > 0;
    if strobe {
        s.controller1.index = 0;
    }
}
