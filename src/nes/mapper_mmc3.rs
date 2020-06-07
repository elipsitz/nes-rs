use super::cartridge::Cartridge;
use super::mapper::{Mapper, MirrorMode, translate_vram};

pub struct MapperMmc3 {
    cart: Cartridge,
    ram: [u8; 8192],
    vram: [u8; 2048],

    reg_bank_select: u8,
    reg_bank_data: [u8; 8],
    reg_ram_protect: u8,
    mirror_mode: MirrorMode,

    // 4 x 8 KB banks
    offset_prg: [usize; 4],
    // 8 x 1 KB banks
    offset_chr: [usize; 8],

    irq_enabled: bool,
    irq_pending: bool,
    irq_reload: bool,
    irq_counter: u8,
    irq_latch: u8,
    last_a12: bool,
}

fn get_bank_offset(total_size: usize, bank_size: usize, bank: i32) -> usize {
    let banks = (total_size / bank_size) as i32;
    let bank = bank.rem_euclid(banks) as usize;
    bank * bank_size
}

impl MapperMmc3 {
    pub fn new(cart: Cartridge) -> MapperMmc3 {
        let mut mapper = MapperMmc3 {
            cart,
            ram: [0; 8192],
            vram: [0; 2048],

            mirror_mode: MirrorMode::MirrorHorizontal,
            reg_bank_select: 0,
            reg_bank_data: [0; 8],
            reg_ram_protect: 0,

            offset_prg: [0; 4],
            offset_chr: [0; 8],

            irq_enabled: false,
            irq_pending: false,
            irq_reload: false,
            irq_counter: 0,
            irq_latch: 0,
            last_a12: false,
        };
        mapper.update_banks();
        mapper
    }

    fn update_banks(&mut self) {
        let prg_mode = self.reg_bank_select & 0b01000000;
        let prg_banks = if prg_mode == 0 {
            [self.reg_bank_data[6] as i8, self.reg_bank_data[7] as i8, -2i8, -1i8]
        } else {
            [-2i8, self.reg_bank_data[7] as i8, self.reg_bank_data[6] as i8, -1i8]
        };
        let prg_len = self.cart.prg_rom.len();
        for i in 0..4 {
            self.offset_prg[i] = get_bank_offset(prg_len, 8 * 1024, prg_banks[i] as i32);
        }

        let chr_mode = self.reg_bank_select & 0b10000000;
        let chr_banks = if chr_mode == 0 {
            [
                self.reg_bank_data[0] & 0xFE, (self.reg_bank_data[0] & 0xFE) | 1,
                self.reg_bank_data[1] & 0xFE, (self.reg_bank_data[1] & 0xFE) | 1,
                self.reg_bank_data[2], self.reg_bank_data[3],
                self.reg_bank_data[4], self.reg_bank_data[5],
            ]
        } else {
            [
                self.reg_bank_data[2], self.reg_bank_data[3],
                self.reg_bank_data[4], self.reg_bank_data[5],
                self.reg_bank_data[0] & 0xFE, (self.reg_bank_data[0] & 0xFE) | 1,
                self.reg_bank_data[1] & 0xFE, (self.reg_bank_data[1] & 0xFE) | 1,
            ]
        };
        let chr_len = self.cart.chr_rom.len();
        for i in 0..8 {
            self.offset_chr[i] = get_bank_offset(chr_len, 1024, chr_banks[i] as i32);
        }
    }

    fn write_register(&mut self, addr: u16, val: u8) {
        match addr {
            // Bank Select
            0x8000..=0x9FFF if (addr % 2 == 0) => self.reg_bank_select = val,
            // Bank Data
            0x8000..=0x9FFF if (addr % 2 == 1) => {
                let bank = self.reg_bank_select & 0b111;
                self.reg_bank_data[bank as usize] = val;
                self.update_banks();
            },
            // Mirroring
            0xA000..=0xBFFF if (addr % 2 == 0) => {
                self.mirror_mode = if val & 0x1 == 0 {
                    MirrorMode::MirrorVertical
                } else {
                    MirrorMode::MirrorHorizontal
                };
            },
            // PRG RAM Protect
            0xA000..=0xBFFF if (addr % 2 == 1) => self.reg_ram_protect = val,
            // IRQ latch
            0xC000..=0xDFFF if (addr % 2 == 0) => self.irq_latch = val,
            // IRQ reload
            0xC000..=0xDFFF if (addr % 2 == 1) => self.irq_reload = true,
            // IRQ disable
            0xE000..=0xFFFF if (addr % 2 == 0) => {
                self.irq_enabled = false;
                self.irq_pending = true;
            },
            // IRQ enable
            0xE000..=0xFFFF if (addr % 2 == 1) => self.irq_enabled = true,
            _ => unreachable!()
        }
    }

    fn check_a12(&mut self, addr: u16) {
        // Check if PPR A12 is high, for counting scanlines.
        // Once A12 goes high, it decrements the IRQ counter. It's filtered though --
        // it's after A12 goes high for the first time after 2 CPU clock cycles.
        // But we don't track that, so let's just assume it's when it goes high at all.
        let a12 = (addr & 0b1000000000000) > 0;

        if a12 && !self.last_a12 {
            if self.irq_counter == 0 || self.irq_reload {
                self.irq_counter = self.irq_latch;
                self.irq_reload = false;
            } else {
                self.irq_counter -= 1;
            }

            if self.irq_counter == 0 && self.irq_enabled {
                self.irq_pending = true;
            }
        }
        self.last_a12 = a12;
    }
}

impl Mapper for MapperMmc3 {


    fn peek(&mut self, addr: u16) -> u8 {
        match addr {
            // PPU
            0x0000..=0x1FFF => {
                self.check_a12(addr);

                let bank = ((addr & 0xFC00) >> 10) as usize;
                let offset = (addr & 0x3FF) as usize;
                let location = self.offset_chr[bank] + offset;
                self.cart.chr_rom[location]
            }
            0x2000..=0x3EFF => self.vram[translate_vram(self.mirror_mode, addr)],

            // CPU
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize],
            0x8000..=0xFFFF => {
                let bank = ((addr & 0x6000) >> 13) as usize;
                let offset = (addr & 0x1FFF) as usize;
                let location = self.offset_prg[bank] + offset;
                self.cart.prg_rom[location]
            }
            _ => 0
        }
    }

    fn poke(&mut self, addr: u16, val: u8) {
        match addr {
            // PPU
            0x0000..=0x1FFF => {
                self.check_a12(addr);

                let bank = ((addr & 0xF800) >> 11) as usize;
                let offset = (addr & 0x7FF) as usize;
                let location = self.offset_chr[bank] + offset;
                self.cart.chr_rom[location] = val;
            }
            0x2000..=0x3EFF => self.vram[translate_vram(self.mirror_mode, addr)] = val,

            // CPU
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize] = val,
            0x8000..=0xFFFF => self.write_register(addr, val),
            _ => {}
        };
    }

    fn check_irq(&self) -> bool {
        self.irq_pending
    }
}