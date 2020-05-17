use super::cartridge::Cartridge;
use super::mapper;
use super::cpu;

pub struct Nes {
    pub state: State,
}

pub struct State {
    pub ram: [u8; 2048],
    pub cpu: CpuState,
    pub ppu: PpuState,
    pub mapper: Box<dyn mapper::Mapper>,
}

#[derive(Default)]
pub struct CpuState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,

    pub status_c: bool,
    pub status_z: bool,
    pub status_i: bool,
    pub status_d: bool,
    pub status_v: bool,
    pub status_n: bool,

    pub cycles: u64,
}

#[derive(Default)]
pub struct PpuState {
}

impl Nes {
    pub fn new(cart: Cartridge) -> Nes {
        let mut nes = Nes {
            state: State::new(cart),
        };
        nes.state.cpu.pc = 0xC000u16; // XXX: nestest auto mode
        nes
    }

    pub fn run(&mut self) {
        cpu::emulate(&mut self.state, 100);
    }
}

impl CpuState {
    pub fn new() -> CpuState {
        CpuState {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xFD,

            // FIXME: fix status
            status_c: false,
            status_z: false,
            status_i: false,
            status_d: false,
            status_v: false,
            status_n: false,

            cycles: 0,
        }
    }
}

impl State {
    pub fn new(cart: Cartridge) -> State {
        State {
            ram: [0; 2048],
            cpu: CpuState::default(),
            ppu: PpuState::default(),
            mapper: mapper::make_mapper(cart),
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