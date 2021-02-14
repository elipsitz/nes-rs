use super::cartridge::Cartridge;
use super::mapper::{translate_vram, Mapper, MirrorMode};

pub struct MapperNrom {
    cart: Cartridge,
    vram: [u8; 2048],
    mirror_mode: MirrorMode,
}

impl MapperNrom {
    pub fn new(cart: Cartridge) -> MapperNrom {
        let mirror_mode = match cart.mirror_mode {
            0 => MirrorMode::MirrorHorizontal,
            1 => MirrorMode::MirrorVertical,
            _ => panic!("Unsupported cart mirror mode: {}", cart.mirror_mode),
        };
        MapperNrom {
            cart,
            vram: [0; 2048],
            mirror_mode,
        }
    }
}

impl Mapper for MapperNrom {
    fn peek(&mut self, addr: u16) -> u8 {
        match addr {
            // PPU
            0x0000..=0x1FFF => self.cart.chr_rom[addr as usize],
            0x2000..=0x3EFF => self.vram[translate_vram(self.mirror_mode, addr)],

            // CPU
            0x8000..=0xFFFF => {
                let offset = addr - 0x8000;
                let size = self.cart.prg_rom.len() as u16;
                self.cart.prg_rom[(offset % size) as usize]
            }
            _ => 0,
        }
    }

    fn poke(&mut self, addr: u16, val: u8) {
        match addr {
            // PPU
            0x0000..=0x1FFF => self.cart.chr_rom[addr as usize] = val,
            0x2000..=0x3EFF => self.vram[translate_vram(self.mirror_mode, addr)] = val,
            _ => {}
        };
    }
}
