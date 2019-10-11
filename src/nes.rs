use crate::rom::Rom;
use crate::cpu::Cpu;

pub struct Nes {
    rom: Rom,
    cpu: Cpu,
}

impl Nes {
    pub fn new_from_rom(rom: Rom) -> Nes {
        Nes {
            rom,
            cpu: Cpu::new(),
        }
    }
}