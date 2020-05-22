use super::cartridge::Cartridge;
use super::mapper;
use super::cpu;
use super::ppu;

pub struct Nes {
    pub state: State,
}

pub struct State {
    pub ram: [u8; 2048],
    pub cpu: cpu::CpuState,
    pub ppu: ppu::PpuState,
    pub mapper: Box<dyn mapper::Mapper>,
}

impl Nes {
    pub fn new(cart: Cartridge) -> Nes {
        let mut nes = Nes {
            state: State::new(cart),
        };
        nes.state.cpu.cycles = 7;
        nes.state.cpu.pc = 0xC000u16; // XXX: nestest auto mode
        nes
    }

    pub fn run(&mut self) {
        cpu::emulate(&mut self.state, 26555);
    }
}

impl State {
    pub fn new(cart: Cartridge) -> State {
        State {
            ram: [0; 2048],
            cpu: cpu::CpuState::new(),
            ppu: ppu::PpuState::new(),
            mapper: mapper::make_mapper(cart),
        }
    }

    pub fn cpu_peek(&mut self, addr: u16) -> u8 {
        // https://wiki.nesdev.com/w/index.php/CPU_memory_map
        let data = match addr {
            0x0000..=0x17FF => self.ram[(addr & 0x7FF) as usize],
            0x4020..=0xFFFF => self.mapper.peek(addr),
            _ => panic!("out of bounds read")
        };
        self.cpu.cycles += 1;
        // eprintln!("##### read from 0x{:04X}: val: {:02X}. cycle: {}", addr, data, self.cpu.cycles);
        data
    }

    pub fn cpu_poke(&mut self, addr: u16, val: u8) {
        // eprintln!("##### store to 0x{:04X}: val: {}. cycle: {}", addr, val, self.cpu.cycles);
        // https://wiki.nesdev.com/w/index.php/CPU_memory_map
        match addr {
            0x0000..=0x17FF => self.ram[(addr & 0x7FF) as usize] = val,
            0x4020..=0xFFFF => self.mapper.poke(addr, val),
            _ => panic!("out of bounds read")
        }
        self.cpu.cycles += 1;
    }
}