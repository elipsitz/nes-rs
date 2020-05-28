use super::cartridge::Cartridge;

pub trait Mapper {
    fn peek(&mut self, addr: u16) -> u8;
    fn poke(&mut self, addr: u16, val: u8);
}

pub fn make_mapper(cart: Cartridge) -> Box<dyn Mapper> {
    match cart.mapper_id {
        0 => Box::new(super::mapper_nrom::MapperNrom::new(cart)),
        _ => panic!("Unknown mapper ID: {}", cart.mapper_id)
    }
}