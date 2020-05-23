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

    background_data: u64,

    // Scrolling registers
    v: u16,
    t: u16,
    x: u16,
    w: u8,

    // PPUCTRL
    flag_vram_increment: u8,
    flag_sprite_table_addr: u8,
    flag_background_table_addr: u8,
    flag_sprite_size: u8,
    flag_master_slave: u8,
    flag_generate_nmi: bool,

    // PPUMASK
    flag_grayscale: bool,
    flag_show_sprites_left: bool,
    flag_show_background_left: bool,
    flag_render_sprites: bool,
    flag_render_background: bool,
    flag_emphasize_red: bool,
    flag_emphasize_green: bool,
    flag_emphasize_blue: bool,
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
        if ppu.scanline == 261 && ppu.tick == 1 {
            // Pre-render.
            ppu.vblank = 0;
            ppu.frame_buffer.clear();
        }

        let rendering_enabled = ppu.flag_render_sprites || ppu.flag_render_background;
        if ppu.scanline < 240 && rendering_enabled {
            render_pixel();
        }

        if ppu.scanline <= 239 || ppu.scanline == 261 {
            // Pre-render and visible scanlines.
            if (ppu.tick >= 1 && ppu.tick <= 256) || (ppu.tick >= 321 && ppu.tick <= 336) {
                if ppu.tick & 0x7 == 1 {
                    fetch_tile();
                }
            }
        }

        // Scanline 240 (post-render) is idle.

        if ppu.scanline == 241 && ppu.tick == 1 {
            // Start of vblank.
            if ppu.flag_generate_nmi {
                s.cpu.pending_interrupt = cpu::InterruptKind::NMI;
            }
            ppu.vblank = 1;
            ppu.frames += 1;
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

fn render_pixel() {

}

fn fetch_tile() {

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
            // t: ...BA.. ........ = d: ......BA
            ppu.t = (ppu.t & 0b1111_0011_1111_1111)
                | (((data & 0b11) as u16) << 10);

            ppu.flag_vram_increment = (data >> 2) & 0x1;
            ppu.flag_sprite_table_addr = (data >> 3) & 0x1;
            ppu.flag_background_table_addr = (data >> 4) & 0x1;
            ppu.flag_sprite_size = (data >> 5) & 0x1;
            ppu.flag_master_slave = (data >> 6) & 0x1;
            ppu.flag_generate_nmi = (data >> 7) & 0x1 > 0;
        }
        1 => {
            // PPUMASK
            ppu.flag_grayscale = (data >> 0) & 0x1 > 0;
            ppu.flag_show_background_left = (data >> 1) & 0x1 > 0;
            ppu.flag_show_sprites_left = (data >> 2) & 0x1 > 0;
            ppu.flag_render_background = (data >> 3) & 0x1 > 0;
            ppu.flag_render_sprites = (data >> 4) & 0x1 > 0;
            ppu.flag_emphasize_red = (data >> 5) & 0x1 > 0;
            ppu.flag_emphasize_green = (data >> 6) & 0x1 > 0;
            ppu.flag_emphasize_blue = (data >> 7) & 0x1 > 0;
        }
        3 => {
            // TODO OAMADDR
        }
        4 => {
            // TODO OAMDATA
        }
        5 => {
            // PPUSCROLL
            // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Register_controls
            if ppu.w == 0 {
                // t: ....... ...HGFED = d: HGFED...
                ppu.t = (ppu.t & 0b1111_1111_1110_0000)
                    | ((data & 0b11111000) as u16 >> 3);
                // x:              CBA = d: .....CBA
                ppu.x = (data & 0b111) as u16;
                ppu.w = 1;
            } else {
                // t: CBA..HG FED..... = d: HGFEDCBA
                ppu.t = (ppu.t & 0b1000_1100_0001_1111)
                    | ((data & 0b0000_0111) as u16) << 12
                    | ((data & 0b1111_1000) as u16) << 2;
                ppu.w = 0;
            }
        }
        6 => {
            // PPUADDR
            if ppu.w == 0 {
                ppu.t = (ppu.t & 0b1000_0000_1111_1111)
                    | ((data & 0b0011_1111) as u16) << 8;
                ppu.w = 1;
            } else {
                ppu.t = (ppu.t & 0xFF00) | (data as u16);
                ppu.v = ppu.t;
                ppu.w = 0;
            }
        }
        7 => {
            // TODO PPUDATA
        }
        _ => {}
    };
}