use super::nes::{State, FRAME_SIZE};
use super::cpu;
use crate::nes::nes::{FRAME_WIDTH, FRAME_DEPTH};

const COLORS: [u32; 64] = [
    0x545454, 0x001e74, 0x081090, 0x300088, 0x440064, 0x5c0030, 0x540400, 0x3c1800,
    0x202a00, 0x083a00, 0x004000, 0x003c00, 0x00323c, 0x000000, 0x000000, 0x000000,
    0x989698, 0x084cc4, 0x3032ec, 0x5c1ee4, 0x8814b0, 0xa01464, 0x982220, 0x783c00,
    0x545a00, 0x287200, 0x087c00, 0x007628, 0x006678, 0x000000, 0x000000, 0x000000,
    0xeceeec, 0x4c9aec, 0x787cec, 0xb062ec, 0xe454ec, 0xec58b4, 0xec6a64, 0xd48820,
    0xa0aa00, 0x74c400, 0x4cd020, 0x38cc6c, 0x38b4cc, 0x3c3c3c, 0x000000, 0x000000,
    0xeceeec, 0xa8ccec, 0xbcbcec, 0xd4b2ec, 0xecaeec, 0xecaed4, 0xecb4b0, 0xe4c490,
    0xccd278, 0xb4de78, 0xa8e290, 0x98e2b4, 0xa0d6e4, 0xa0a2a0, 0x000000, 0x000000,
];

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

    data_buffer: u8,
    latch: u8,
    sprite_overflow: u8,
    sprite0_hit: u8,
    vblank: u8,

    pub palette: [u8; 32],
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
    let mut cycles_left = cycles;
    while cycles_left > 0 {
        if s.ppu.scanline == 261 && s.ppu.tick == 1 {
            // Pre-render.
            s.ppu.vblank = 0;
            s.ppu.frame_buffer.clear();
        }

        let rendering_enabled = s.ppu.flag_render_sprites || s.ppu.flag_render_background;
        if s.ppu.scanline < 240 && rendering_enabled && s.ppu.tick >= 1 && s.ppu.tick <= 256 {
            render_pixel(s);
        }

        if s.ppu.scanline <= 239 || s.ppu.scanline == 261 {
            // Pre-render and visible scanlines.
            if (s.ppu.tick >= 1 && s.ppu.tick <= 256) || (s.ppu.tick >= 321 && s.ppu.tick <= 336) {
                s.ppu.background_data <<= 4;

                if s.ppu.tick & 0x7 == 1 {
                    fetch_tile(s);
                }
            }
        }

        // Scanline 240 (post-render) is idle.

        if s.ppu.scanline == 241 && s.ppu.tick == 1 {
            // Start of vblank.
            if s.ppu.flag_generate_nmi {
                s.cpu.pending_interrupt = cpu::InterruptKind::NMI;
            }
            s.ppu.vblank = 1;
            s.ppu.frames += 1;
        }

        // Increment counters.
        s.ppu.cycles += 1;
        s.ppu.tick += 1;
        if s.ppu.tick == 341 || (s.ppu.scanline == 261 && (s.ppu.frames & 1 > 0) && s.ppu.tick == 340) {
            s.ppu.tick = 0;
            s.ppu.scanline += 1;
            if s.ppu.scanline > 261 {
                s.ppu.scanline = 0;
            }
        }
        cycles_left -= 1;
    }
}

fn render_pixel(s: &mut State) {
    // let bg_pixel = (s.ppu.background_data >> (32 + ((7 - s.ppu.x) * 4)) as u64 & 0xF) as u8 * 100;
    let x = (s.ppu.tick - 1) as usize;
    let y = s.ppu.scanline as usize;
    let mut col = 0;

    if y < 128 {
        let mut addr = 0
            | (y % 8)
            | (x % 128 / 8) << 4
            | (y % 128 / 8) << 8;
        if x >= 128 {
            addr |= 0x1000;
        }
        let lo = s.ppu_peek(addr as u16);
        let hi = s.ppu_peek((addr + 8) as u16);
        col = (((lo << (x % 8) as u8) & 0x80) >> 7) | (((hi << (x % 8) as u8) & 0x80) >> 6);
    }

    let pixel = COLORS[s.ppu.palette[(col & 0x1F) as usize] as usize];

    let frame = &mut s.ppu.frame_buffer.0;
    let i = ((y * FRAME_WIDTH) + x) * FRAME_DEPTH;
    frame[i + 0] = ((pixel & 0xFF0000) >> 16) as u8;
    frame[i + 1] = ((pixel & 0x00FF00) >> 8) as u8;
    frame[i + 2] = ((pixel & 0x0000FF) >> 0) as u8;
    frame[i + 3] = 255;
}

fn fetch_tile(s: &mut State) {
    let nt_addr = 0x2000 | (s.ppu.v & 0x0FFF);
    let nt_data = s.ppu_peek(nt_addr) as u16;
    let at_addr = 0x23C0 | (s.ppu.v & 0x0C00) | ((s.ppu.v >> 4) & 0x38) | ((s.ppu.v >> 2) & 0x07);
    let at_data = s.ppu_peek(at_addr) as u16;

    // process attribute data to select correct tile
    let at_data = ((at_data >> (((s.ppu.v >> 4) & 4) | (s.ppu.v & 2))) & 3) << 2;

    let pattern_addr: u16 = 0
        | ((s.ppu.v >> 12) & 0x7)
        | nt_data << 4
        | (s.ppu.flag_background_table_addr as u16) << 12;

    let mut pattern_lo = s.ppu_peek(pattern_addr) as u16;
    let mut pattern_hi = s.ppu_peek(pattern_addr + 8) as u16;

    let mut bitmap: u64 = 0;
    for _ in 0..8 {
        let pixel_data = at_data | ((pattern_lo & 0x80) >> 7) | ((pattern_hi & 0x80) >> 6);
        pattern_lo <<= 1;
        pattern_hi <<= 1;
        bitmap = (bitmap << 4) | (pixel_data as u64);
    }
    s.ppu.background_data |= bitmap;
}

pub fn peek_register(s: &mut State, register: u16) -> u8 {
    s.ppu.latch = match register {
        2 => {
            // PPUSTATUS
            let data = (s.ppu.latch & 0x1F)
                | (s.ppu.sprite_overflow) << 5
                | (s.ppu.sprite0_hit) << 6
                | (s.ppu.vblank) << 7;

            s.ppu.vblank = 0;
            s.ppu.w = 0;
            data
        }
        4 => {
            // TODO OAMDATA
            0
        }
        7 => {
            // PPUDATA
            let mut data = s.ppu_peek(s.ppu.v);
            if s.ppu.v <= 0x3EFF {
                // buffer this read
                std::mem::swap(&mut data, &mut s.ppu.data_buffer);
            } else {
                s.ppu.data_buffer = s.ppu_peek(s.ppu.v - 0x1000);
            }

            s.ppu.v += if s.ppu.flag_vram_increment == 0 { 1 } else { 32 };
            data
        }
        _ => s.ppu.latch
    };
    s.ppu.latch
}

pub fn poke_register(s: &mut State, register: u16, data: u8) {
    s.ppu.latch = data;
    match register {
        0 => {
            // PPUCTRL
            // t: ...BA.. ........ = d: ......BA
            s.ppu.t = (s.ppu.t & 0b1111_0011_1111_1111)
                | (((data & 0b11) as u16) << 10);

            s.ppu.flag_vram_increment = (data >> 2) & 0x1;
            s.ppu.flag_sprite_table_addr = (data >> 3) & 0x1;
            s.ppu.flag_background_table_addr = (data >> 4) & 0x1;
            s.ppu.flag_sprite_size = (data >> 5) & 0x1;
            s.ppu.flag_master_slave = (data >> 6) & 0x1;
            s.ppu.flag_generate_nmi = (data >> 7) & 0x1 > 0;
        }
        1 => {
            // PPUMASK
            s.ppu.flag_grayscale = (data >> 0) & 0x1 > 0;
            s.ppu.flag_show_background_left = (data >> 1) & 0x1 > 0;
            s.ppu.flag_show_sprites_left = (data >> 2) & 0x1 > 0;
            s.ppu.flag_render_background = (data >> 3) & 0x1 > 0;
            s.ppu.flag_render_sprites = (data >> 4) & 0x1 > 0;
            s.ppu.flag_emphasize_red = (data >> 5) & 0x1 > 0;
            s.ppu.flag_emphasize_green = (data >> 6) & 0x1 > 0;
            s.ppu.flag_emphasize_blue = (data >> 7) & 0x1 > 0;
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
            if s.ppu.w == 0 {
                // t: ....... ...HGFED = d: HGFED...
                s.ppu.t = (s.ppu.t & 0b1111_1111_1110_0000)
                    | ((data & 0b11111000) as u16 >> 3);
                // x:              CBA = d: .....CBA
                s.ppu.x = (data & 0b111) as u16;
                s.ppu.w = 1;
            } else {
                // t: CBA..HG FED..... = d: HGFEDCBA
                s.ppu.t = (s.ppu.t & 0b1000_1100_0001_1111)
                    | ((data & 0b0000_0111) as u16) << 12
                    | ((data & 0b1111_1000) as u16) << 2;
                s.ppu.w = 0;
            }
        }
        6 => {
            // PPUADDR
            if s.ppu.w == 0 {
                s.ppu.t = (s.ppu.t & 0b1000_0000_1111_1111)
                    | ((data & 0b0011_1111) as u16) << 8;
                s.ppu.w = 1;
            } else {
                s.ppu.t = (s.ppu.t & 0xFF00) | (data as u16);
                s.ppu.v = s.ppu.t;
                s.ppu.w = 0;
            }
        }
        7 => {
            // PPUDATA
            s.ppu_poke(s.ppu.v, data);
            s.ppu.v += if s.ppu.flag_vram_increment == 0 { 1 } else { 32 };
        }
        _ => {}
    };
}