use super::cartridge::Cartridge;
use super::mapper;
use super::cpu;
use super::ppu;

pub const FRAME_DEPTH: usize = 4;
pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;
pub const FRAME_SIZE: usize = FRAME_DEPTH * FRAME_WIDTH * FRAME_HEIGHT;

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
        nes.state.cpu.pc = cpu::vector_reset(&mut nes.state);
        println!("[nes] Reset to pc = {:#04X}", nes.state.cpu.pc);
        // nes.state.cpu.pc = 0xC000u16; // nestest auto mode
        nes
    }

    pub fn emulate_frame(&mut self) {
        let start_frame = self.state.ppu.frames;
        while self.state.ppu.frames == start_frame {
            let cycles = cpu::emulate(&mut self.state, 1);
            ppu::emulate(&mut self.state, cycles * 3);
        }
    }

    pub fn get_frame_buffer(&self) -> &[u8; FRAME_SIZE] {
        &self.state.ppu.frame_buffer.0
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
            0x0000..=0x1FFF => self.ram[(addr & 0x7FF) as usize],
            0x2000..=0x3FFF => ppu::peek_register(self, addr & 0x7),
            0x4000..=0x401F => /* TODO: APU, input */ 0,
            _ /*0x4020..=0xFFFF*/ => self.mapper.peek(addr),
        };
        self.cpu.cycles += 1;
        // eprintln!("##### read from 0x{:04X}: val: {:02X}. cycle: {}", addr, data, self.cpu.cycles);
        data
    }

    pub fn cpu_poke(&mut self, addr: u16, val: u8) {
        // eprintln!("##### store to 0x{:04X}: val: {}. cycle: {}", addr, val, self.cpu.cycles);
        // https://wiki.nesdev.com/w/index.php/CPU_memory_map
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x7FF) as usize] = val,
            0x2000..=0x3FFF => ppu::poke_register(self, addr & 0x7, val),
            0x4000..=0x401F => {} /* TODO: APU, input */
            _ /* 0x4020..=0xFFFF */ => self.mapper.poke(addr, val),
        }
        self.cpu.cycles += 1;
    }

    pub fn ppu_peek(&mut self, addr: u16) -> u8 {
        // https://wiki.nesdev.com/w/index.php/PPU_memory_map
        match addr {
            0x3F00..=0x3FFF => self.ppu.palette[(addr & 0x1F) as usize],
            _ => self.mapper.peek(addr),
        }
    }

    pub fn ppu_poke(&mut self, addr: u16, data: u8) {
        // https://wiki.nesdev.com/w/index.php/PPU_memory_map
        match addr {
            0x3F00..=0x3FFF => { self.ppu.palette[(addr & 0x1F) as usize] = data },
            _ => self.mapper.poke(addr, data),
        }
    }
}