use super::cartridge::Cartridge;

pub trait Mapper {
    fn peek(&mut self, addr: u16) -> u8;
    fn poke(&mut self, addr: u16, val: u8);

    fn check_irq(&self) -> bool {
        false
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum MirrorMode {
    MirrorHorizontal,
    MirrorVertical,
    MirrorSingleA,
    MirrorSingleB,
    MirrorFour,
}

pub fn translate_vram(mode: MirrorMode, addr: u16) -> usize {
    (match mode {
        MirrorMode::MirrorHorizontal => (addr & 0x3FF) | ((addr & 0x800) >> 1),
        MirrorMode::MirrorVertical => addr & 0x7FF,
        MirrorMode::MirrorSingleA => addr & 0x3FF,
        MirrorMode::MirrorSingleB => 0x400 | (addr & 0x3FF),
        _ => panic!("Unsupported mirror mode: {:?}")
    }) as usize
}

pub fn make_mapper(cart: Cartridge) -> Box<dyn Mapper> {
    match cart.mapper_id {
        0 => Box::new(super::mapper_nrom::MapperNrom::new(cart)),
        1 => Box::new(super::mapper_mmc1::MapperMmc1::new(cart)),
        4 => Box::new(super::mapper_mmc3::MapperMmc3::new(cart)),
        _ => panic!("Unknown mapper ID: {}", cart.mapper_id)
    }
}