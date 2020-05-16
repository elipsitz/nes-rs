use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::mapper;

pub struct Nes {
    rom: Cartridge,
    ram: [u8; 2048],
    cpu: Cpu,
    ppu: Ppu,
    mapper: Box<dyn mapper::Mapper>,
}

impl Nes {
    pub fn new_from_rom(rom: Cartridge) -> Nes {
        Nes {
            rom,
            ram: [0; 2048],
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            mapper: Box::new(mapper::Mapper0::new()),
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        // https://wiki.nesdev.com/w/index.php/CPU_memory_map
        match addr {
            0x0000..=0x17FF => self.ram[(addr & 0x7FF) as usize],
            0x4020..=0xFFFF => self.mapper.read(addr),
            _ => panic!("out of bounds read")
        }
    }

    pub fn cpu_write(&mut self, addr: u16, val: u8) {
        // https://wiki.nesdev.com/w/index.php/CPU_memory_map
        match addr {
            0x0000..=0x17FF => self.ram[(addr & 0x7FF) as usize] = val,
            0x4020..=0xFFFF => self.mapper.write(addr, val),
            _ => panic!("out of bounds read")
        }
    }
}