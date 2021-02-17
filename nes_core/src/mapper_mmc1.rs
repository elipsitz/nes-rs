use super::cartridge::Cartridge;
use super::mapper::{translate_vram, Mapper, MirrorMode};

pub struct MapperMmc1 {
    cart: Cartridge,
    ram: [u8; 8192],
    vram: [u8; 2048],

    shift_number: u8,
    shift_data: u8,
    reg_control: u8,
    reg_chr0: u8,
    reg_chr1: u8,
    reg_prg: u8,

    mirror_mode: MirrorMode,
    offset_prg0: usize,
    offset_prg1: usize,
    offset_chr0: usize,
    offset_chr1: usize,
}

impl MapperMmc1 {
    pub fn new(cart: Cartridge) -> MapperMmc1 {
        let mut mapper = MapperMmc1 {
            cart,
            ram: [0; 8192],
            vram: [0; 2048],
            shift_number: 0,
            shift_data: 0,
            reg_control: 0x1F, // ???
            reg_chr0: 0,
            reg_chr1: 0,
            reg_prg: 0,
            mirror_mode: MirrorMode::MirrorHorizontal,
            offset_prg0: 0,
            offset_prg1: 0,
            offset_chr0: 0,
            offset_chr1: 0,
        };
        mapper.update_mapping();
        mapper
    }

    fn update_mapping(&mut self) {
        self.mirror_mode = match self.reg_control & 0b11 {
            0 => MirrorMode::MirrorSingleA,
            1 => MirrorMode::MirrorSingleB,
            2 => MirrorMode::MirrorVertical,
            3 => MirrorMode::MirrorHorizontal,
            _ => unreachable!(),
        };

        let prg_mode = (self.reg_control & 0b01100) >> 2;
        let chr_mode = (self.reg_control & 0b10000) >> 4;

        // TODO: ram chip enable bit?
        match prg_mode {
            0 | 1 => {
                let prg_bank = (16 * 1024) * ((self.reg_prg as usize) & 0b01110);
                self.offset_prg0 = prg_bank;
                self.offset_prg1 = prg_bank + (16 * 1024);
            }
            2 => {
                self.offset_prg0 = 0;
                self.offset_prg1 = (16 * 1024) * ((self.reg_prg as usize) & 0b01111);
            }
            3 => {
                self.offset_prg0 = (16 * 1024) * ((self.reg_prg as usize) & 0b01111);
                self.offset_prg1 = self.cart.prg_rom.len() - (16 * 1024);
            }
            _ => unreachable!(),
        }
        self.offset_prg0 %= self.cart.prg_rom.len();
        self.offset_prg1 %= self.cart.prg_rom.len();

        match chr_mode {
            0 => {
                self.offset_chr0 = ((self.reg_chr0 as usize) & 0b11110) * (4 * 1024);
                self.offset_chr1 = self.offset_chr0 + (4 * 1024);
            }
            1 => {
                self.offset_chr0 = (self.reg_chr0 as usize) * (4 * 1024);
                self.offset_chr1 = (self.reg_chr1 as usize) * (4 * 1024);
            }
            _ => unreachable!(),
        }
        self.offset_chr0 %= self.cart.chr_rom.len();
        self.offset_chr1 %= self.cart.chr_rom.len();
    }

    fn handle_control(&mut self, register: u16, data: u8) {
        match register {
            0 => self.reg_control = data,
            1 => self.reg_chr0 = data,
            2 => self.reg_chr1 = data,
            3 => self.reg_prg = data,
            _ => {}
        }
        self.update_mapping();
    }
}

impl Mapper for MapperMmc1 {
    fn peek(&mut self, addr: u16) -> u8 {
        match addr {
            // PPU
            0x0000..=0x0FFF => self.cart.chr_rom[self.offset_chr0 + (addr & 0xFFF) as usize],
            0x1000..=0x1FFF => self.cart.chr_rom[self.offset_chr1 + (addr & 0xFFF) as usize],
            0x2000..=0x3EFF => self.vram[translate_vram(self.mirror_mode, addr)],

            // CPU 3FFF
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize],
            0x8000..=0xBFFF => self.cart.prg_rom[self.offset_prg0 + (addr & 0x3FFF) as usize],
            0xC000..=0xFFFF => self.cart.prg_rom[self.offset_prg1 + (addr & 0x3FFF) as usize],
            _ => 0,
        }
    }

    fn poke(&mut self, addr: u16, val: u8) {
        match addr {
            // PPU
            0x0000..=0x0FFF => self.cart.chr_rom[self.offset_chr0 + (addr & 0xFFF) as usize] = val,
            0x1000..=0x1FFF => self.cart.chr_rom[self.offset_chr1 + (addr & 0xFFF) as usize] = val,
            0x2000..=0x3EFF => self.vram[translate_vram(self.mirror_mode, addr)] = val,

            // CPU
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize] = val,
            0x8000..=0xFFFF => {
                // TODO ignore consecutive writes
                if val & 0x80 > 0 {
                    self.shift_data = 0;
                    self.shift_number = 0;
                    self.reg_control |= 0x0C;
                    self.update_mapping();
                } else {
                    self.shift_data |= (val & 0x1) << self.shift_number;
                    self.shift_number += 1;

                    if self.shift_number == 5 {
                        let register = (addr >> 13) & 0b11;
                        self.handle_control(register, self.shift_data);
                        self.shift_number = 0;
                        self.shift_data = 0;
                    }
                }
            }
            _ => {}
        };
    }
}
