pub trait Mapper {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

pub struct Mapper0 {
}

impl Mapper for Mapper0 {
    fn read(&mut self, addr: u16) -> u8 {
        return 0;
    }

    fn write(&mut self, addr: u16, val: u8) {
    }
}

impl Mapper0 {
    pub fn new() -> Mapper0 {
        Mapper0 {
        }
    }
}