use super::cartridge::Cartridge;

pub trait Mapper {
    fn peek(&mut self, addr: u16) -> u8;
    fn poke(&mut self, addr: u16, val: u8);
}

pub fn make_mapper(cart: Cartridge) -> Box<dyn Mapper> {
    match cart.mapper_id {
        0 => Box::new(Mapper0::new(cart)),
        _ => panic!("Unknown mapper ID: {}", cart.mapper_id)
    }
}

struct Mapper0 {
    cart: Cartridge,
    vram: [u8; 2048],
}

impl Mapper0 {
    fn translate_vram(&self, addr: u16) -> u16 {
        if self.cart.mirror_mode == 0 {
            // Horizontal mirroring.
            (addr & 0x3FF) | ((addr & 0x800) >> 1)
        } else {
            // Vertical mirroring.
            addr & 0x7FF
        }
    }

    fn new(cart: Cartridge) -> Mapper0 {
        assert!(cart.mirror_mode == 0 || cart.mirror_mode == 1);
        Mapper0 {
            cart,
            vram: [0; 2048],
        }
    }
}

impl Mapper for Mapper0 {
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