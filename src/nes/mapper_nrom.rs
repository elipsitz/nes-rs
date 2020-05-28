use super::cartridge::Cartridge;
use super::mapper::Mapper;

pub struct MapperNrom {
    cart: Cartridge,
    vram: [u8; 2048],
}

impl MapperNrom {
    fn translate_vram(&self, addr: u16) -> u16 {
        if self.cart.mirror_mode == 0 {
            // Horizontal mirroring.
            (addr & 0x3FF) | ((addr & 0x800) >> 1)
        } else {
            // Vertical mirroring.
            addr & 0x7FF
        }
    }

    pub fn new(cart: Cartridge) -> MapperNrom {
        assert!(cart.mirror_mode == 0 || cart.mirror_mode == 1);
        MapperNrom {
            cart,
            vram: [0; 2048],
        }
    }
}

impl Mapper for MapperNrom {
    fn peek(&mut self, addr: u16) -> u8 {
        match addr {
            // PPU
            0x0000..=0x1FFF => self.cart.chr_rom[addr as usize],
            0x2000..=0x3EFF => self.vram[self.translate_vram(addr) as usize],

            // CPU
            0x8000..=0xFFFF => {
                let offset = addr - 0x8000;
                let size = self.cart.prg_rom.len() as u16;
                self.cart.prg_rom[(offset % size) as usize]
            }
            _ => 0
        }
    }

    fn poke(&mut self, addr: u16, val: u8) {
        match addr {
            // PPU
            0x0000..=0x1FFF => self.cart.chr_rom[addr as usize] = val,
            0x2000..=0x3EFF => self.vram[self.translate_vram(addr) as usize] = val,
            _ => {}
        };
    }
}