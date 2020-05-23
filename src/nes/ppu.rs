use super::nes::{State, FRAME_SIZE};
use super::cpu;

pub struct FrameBuffer(pub [u8; FRAME_SIZE]);

impl Default for FrameBuffer {
    fn default() -> FrameBuffer {
        FrameBuffer([0; FRAME_SIZE])
    }
}

impl FrameBuffer {
    fn clear(&mut self) {
        for i in 0..self.0.len() {
            self.0[i] = 0;
        }
    }
}

#[derive(Default)]
pub struct PpuState {
    scanline: u16,
    tick: u16,
    pub frames: u64,
    cycles: u64,

    pub frame_buffer: FrameBuffer,

    latch: u8,
    sprite_overflow: u8,
    sprite0_hit: u8,
    vblank: u8,

    // PPUSCROLL/PPUADDR write index
    w: u8,

    // PPUCTRL
    flag_nametable_base: u8,
    flag_vram_increment: u8,
    flag_sprite_table_addr: u8,
    flag_background_table_addr: u8,
    flag_sprite_size: u8,
    flag_master_slave: u8,
    flag_generate_nmi: u8,

    // PPUMASK
    flag_grayscale: u8,
    flag_show_sprites_left: u8,
    flag_show_background_left: u8,
    flag_render_sprites: u8,
    flag_render_background: u8,
    flag_emphasize_red: u8,
    flag_emphasize_green: u8,
    flag_emphasize_blue: u8,
}

impl PpuState {
    pub fn new() -> PpuState {
        PpuState::default()
    }
}

pub fn emulate(s: &mut State, cycles: u64) {
    let ppu = &mut s.ppu;
    let mut cycles_left = cycles;
    while cycles_left > 0 {
        match ppu.scanline {
            261 => {
                // Pre-render scanline.
                if ppu.tick == 1 {
                    ppu.vblank = 0;
                    ppu.frame_buffer.clear();
                }
            }
            0..=239 => {
                // Visible scanlines.
                let x = (ppu.scanline as u64) * (ppu.tick as u64);
                ppu.frame_buffer.0[x as usize] = (x & 0xFF) as u8;
            }
            240 => {
                // Post-render scanline.
            }
            241 => {
                // Start of vblank
                if ppu.tick == 1 {
                    if ppu.flag_generate_nmi > 0 {
                        s.cpu.pending_interrupt = cpu::InterruptKind::NMI;
                    }
                    ppu.vblank = 1;
                    ppu.frames += 1;
                }
            }
            242..=260 => { /* vblank */ }
            _ => {}
        }

        // Increment counters.
        ppu.cycles += 1;
        ppu.tick += 1;
        if ppu.tick == 341 || (ppu.scanline == 261 && (ppu.frames & 1 > 0) && ppu.tick == 340) {
            ppu.tick = 0;
            ppu.scanline += 1;
            if ppu.scanline > 261 {
                ppu.scanline = 0;
            }
        }
        cycles_left -= 1;
    }
}

pub fn peek_register(s: &mut State, register: u16) -> u8 {
    let ppu = &mut s.ppu;
    ppu.latch = match register {
        2 => {
            // PPUSTATUS
            let data = (ppu.latch & 0x1F)
                | (ppu.sprite_overflow) << 5
                | (ppu.sprite0_hit) << 6
                | (ppu.vblank) << 7;

            ppu.vblank = 0;
            ppu.w = 0;
            data
        }
        4 => {
            // TODO OAMDATA
            0
        }
        7 => {
            // TODO PPUDATA
            0
        }
        _ => ppu.latch
    };
    ppu.latch
}

pub fn poke_register(s: &mut State, register: u16, data: u8) {
    let ppu = &mut s.ppu;
    ppu.latch = data;
    match register {
        0 => {
            // PPUCTRL
            ppu.flag_nametable_base = (data >> 0) & 0x3;
            ppu.flag_vram_increment = (data >> 2) & 0x1;
            ppu.flag_sprite_table_addr = (data >> 3) & 0x1;
            ppu.flag_background_table_addr = (data >> 4) & 0x1;
            ppu.flag_sprite_size = (data >> 5) & 0x1;
            ppu.flag_master_slave = (data >> 6) & 0x1;
            ppu.flag_generate_nmi = (data >> 7) & 0x1;
        }
        1 => {
            // PPUMASK
            ppu.flag_grayscale = (data >> 0) & 0x1;
            ppu.flag_show_background_left = (data >> 1) & 0x1;
            ppu.flag_show_sprites_left = (data >> 2) & 0x1;
            ppu.flag_render_background = (data >> 3) & 0x1;
            ppu.flag_render_sprites = (data >> 4) & 0x1;
            ppu.flag_emphasize_red = (data >> 5) & 0x1;
            ppu.flag_emphasize_green = (data >> 6) & 0x1;
            ppu.flag_emphasize_blue = (data >> 7) & 0x1;
        }
        3 => {
            // TODO OAMADDR
        }
        4 => {
            // TODO OAMDATA
        }
        5 => {
            // TODO PPUSCROLL
            // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Register_controls
        }
        6 => {
            // TODO PPUADDR
        }
        7 => {
            // TODO PPUDATA
        }
        _ => {}
    };
}