use super::cartridge::Cartridge;

pub trait Mapper {
    fn peek(&mut self, addr: u16) -> u8;
    fn poke(&mut self, addr: u16, val: u8);
}

pub fn make_mapper(cart: Cartridge) -> Box<dyn Mapper> {
    match cart.mapper_id {
        0 => Box::new(Mapper0 { cart }),
        _ => panic!("Unknown mapper ID: {}", cart.mapper_id)
    }
}

pub struct Mapper0 {
    cart: Cartridge
}

impl Mapper for Mapper0 {
    fn peek(&mut self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                let offset = addr - 0x8000;
                let size = self.cart.prg_rom.len() as u16;
                self.cart.prg_rom[(offset % size) as usize]
            }
            _ => 0
        }
    }

    fn poke(&mut self, _addr: u16, _val: u8) {
    }
}