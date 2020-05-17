use crate::nes::State;
use crate::cartridge::Cartridge;

pub trait Mapper {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

pub fn make_mapper(cart: Cartridge) -> Box<dyn Mapper> {
    Box::new(Mapper0 {
        cart
    })
}

pub struct Mapper0 {
    cart: Cartridge
}

impl Mapper for Mapper0 {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let offset = addr - 0x8000;
                let size = self.cart.prg_rom.len() as u16;
                self.cart.prg_rom[(offset % size) as usize]
            }
            _ => 0
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
    }
}