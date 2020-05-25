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

#[derive(Copy, Clone)]
struct SpriteBufferData {
    id: u8,
    color: u8,
    priority: bool,
    sprite0: bool,
}

impl Default for SpriteBufferData {
    fn default() -> Self {
        SpriteBufferData {
            id: 0xFF,
            color: 0,
            priority: false,
            sprite0: false,
        }
    }
}

pub struct PpuState {
    scanline: u16,
    tick: u16,
    pub frames: u64,
    cycles: u64,

    pub frame_buffer: [u8; FRAME_SIZE],

    is_rendering: bool,
    data_buffer: u8,
    latch: u8,
    sprite_overflow: u8,
    sprite0_hit: bool,
    vblank: u8,

    oam_addr: usize,
    oam_1: [u8; 256],
    oam_2: [u8; 32],
    sprite_eval_n: usize,
    sprite_eval_m: usize,
    sprite_eval_read: u8,
    sprite_eval_scanline_count: usize,
    sprite_eval_has_sprite0: bool, // Whether sprite0 is at oam_2[0]
    sprite_buffer: [SpriteBufferData; 256], // Sprite scanline buffer.

    pub palette: [u8; 32],
    bg_data: [u8; 24],
    bg_data_index: usize,

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
        PpuState {
            scanline: 0,
            tick: 0,
            frames: 0,
            cycles: 0,
            frame_buffer: [0; FRAME_SIZE],
            is_rendering: false,
            data_buffer: 0,
            latch: 0,
            sprite_overflow: 0,
            sprite0_hit: false,
            vblank: 0,
            oam_addr: 0,
            oam_1: [0; 256],
            oam_2: [0; 32],
            sprite_eval_n: 0,
            sprite_eval_m: 0,
            sprite_eval_read: 0,
            sprite_eval_scanline_count: 0,
            sprite_eval_has_sprite0: false,
            sprite_buffer: [SpriteBufferData::default(); 256],
            palette: [0; 32],
            bg_data: [0; 24],
            bg_data_index: 0,
            v: 0,
            t: 0,
            x: 0,
            w: 0,
            flag_vram_increment: 0,
            flag_sprite_table_addr: 0,
            flag_background_table_addr: 0,
            flag_sprite_size: 0,
            flag_master_slave: 0,
            flag_generate_nmi: false,
            flag_grayscale: false,
            flag_show_sprites_left: false,
            flag_show_background_left: false,
            flag_render_sprites: false,
            flag_render_background: false,
            flag_emphasize_red: false,
            flag_emphasize_green: false,
            flag_emphasize_blue: false
        }
    }
}

pub fn emulate(s: &mut State, cycles: u64) {
    let mut cycles_left = cycles;
    while cycles_left > 0 {
        let rendering_enabled = s.ppu.flag_render_sprites || s.ppu.flag_render_background;

        if s.ppu.scanline == 261 {
            // Pre-render.
            if s.ppu.tick == 1 {
                s.ppu.sprite0_hit = false;
                s.ppu.vblank = 0;
                s.ppu.is_rendering = true;
            }
            if s.ppu.tick == 304 && rendering_enabled {
                // XXX:
                // copy vertical scroll bits
                // v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
                s.ppu.v = (s.ppu.v & 0x841F) | (s.ppu.t & 0x7BE0);
            }
        }

        if (s.ppu.scanline <= 239 || s.ppu.scanline == 261) && rendering_enabled {
            // Pre-render and visible scanlines.
            if (s.ppu.tick >= 1 && s.ppu.tick <= 256) || (s.ppu.tick >= 321 && s.ppu.tick <= 336) {
                s.ppu.bg_data_index += 1;
                s.ppu.bg_data_index %= s.ppu.bg_data.len();

                if s.ppu.tick & 0x7 == 1 {
                    fetch_tile(s);
                }
            }
        }

        if s.ppu.scanline < 240 && rendering_enabled {
            if s.ppu.tick >= 1 && s.ppu.tick <= 256 {
                render_pixel(s);
            }

            sprite_evaluation(s);

            // Update scrolling.
            if s.ppu.tick == 256 {
                increment_scroll_y(&mut s.ppu);
            } else if s.ppu.tick == 257 {
                // copy horizontal bits from t to v
                // v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
                s.ppu.v = (s.ppu.v & 0xFBE0) | (s.ppu.t & 0x41F);
            } else if ((s.ppu.tick >= 321 && s.ppu.tick <= 336) || (s.ppu.tick >= 1 && s.ppu.tick <= 256)) && (s.ppu.tick % 8 == 0) {
                increment_scroll_x(&mut s.ppu);
            }
        }

        // Scanline 240 (post-render) is idle.

        if s.ppu.scanline == 241 && s.ppu.tick == 1 {
            // Start of vblank.
            if s.ppu.flag_generate_nmi {
                s.cpu.pending_interrupt = cpu::InterruptKind::NMI;
            }
            s.ppu.is_rendering = false;
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

fn sprite_evaluation(s: &mut State) {
    match s.ppu.tick {
        1 => {
            // Ticks 1-64: clear secondary OAM.
            // But secondary OAM is fully internal state, so just do it all at once.
            for i in 0..32 {
                s.ppu.oam_2[i] = 0xFF;
            }

            s.ppu.sprite_eval_n = 0;
            s.ppu.sprite_eval_m = 0;
            s.ppu.sprite_eval_scanline_count = 0;
            s.ppu.sprite_eval_has_sprite0 = false;
        }
        65..=256 if (s.ppu.sprite_eval_n < 64 && s.ppu.sprite_eval_scanline_count < 8) => {
            // Ticks 65-256: fetch sprite data from primary OAM into secondary OAM.
            if s.ppu.tick & 0x1 == 1 {
                // Primary OAM read
                let index = s.ppu.sprite_eval_n * 4 + s.ppu.sprite_eval_m;
                s.ppu.sprite_eval_read = s.ppu.oam_1[index];
            } else {
                // Secondary OAM write
                let oam_2_addr = 4 * s.ppu.sprite_eval_scanline_count + s.ppu.sprite_eval_m;
                s.ppu.oam_2[oam_2_addr] = s.ppu.sprite_eval_read;
                match s.ppu.sprite_eval_m {
                    0 => {
                        // Y coordinate: is this sprite in range?
                        let sprite_height = 8 << s.ppu.flag_sprite_size;
                        let top = s.ppu.sprite_eval_read;
                        let bottom = s.ppu.sprite_eval_read + sprite_height;
                        if s.ppu.scanline >= (top as u16) && s.ppu.scanline < (bottom as u16) {
                            // Sprite in range.
                            s.ppu.sprite_eval_m += 1;
                        } else {
                            // Sprite not in range.
                            s.ppu.sprite_eval_n += 1;
                        }
                    }
                    3 => {
                        if s.ppu.sprite_eval_n == 0 {
                            s.ppu.sprite_eval_has_sprite0 = true;
                        }
                        s.ppu.sprite_eval_n += 1;
                        s.ppu.sprite_eval_m = 0;
                        s.ppu.sprite_eval_scanline_count += 1;
                    }
                    _ => { s.ppu.sprite_eval_m += 1; }
                }
            }
            // TODO: do sprite overflow flag
        }
        256 => {
            // Internal: set up scanline buffer state.
            for i in 0..256 {
                s.ppu.sprite_buffer[i] = SpriteBufferData::default();
            }
        }
        257..=320 if (s.ppu.tick & 0x7 == 0) => {
            // Ticks 257-320: fetch selected sprite data from pattern tables.
            let n = ((s.ppu.tick - 257) / 8) as usize;
            let (y_pos, tile, attribute, x_pos) = if n < s.ppu.sprite_eval_scanline_count {
                (
                    s.ppu.oam_2[n * 4 + 0],
                    s.ppu.oam_2[n * 4 + 1],
                    s.ppu.oam_2[n * 4 + 2],
                    s.ppu.oam_2[n * 4 + 3],
                )
            } else {
                (0xFF, 0xFF, 0xFF, 0xFF)
            };

            let mut sprite_table = s.ppu.flag_sprite_table_addr;
            let mut tile_row = s.ppu.scanline - (y_pos as u16);
            let mut tile = tile;
            let flip_vertical = attribute & 0x80 > 0;

            if s.ppu.flag_sprite_size > 0 {
                sprite_table = tile & 0x1;
                tile &= 0xFE;
                if tile_row >= 8 {
                    tile |= (flip_vertical as u8) ^ 0x1;
                    tile_row += 8;
                } else {
                    tile |= flip_vertical as u8;
                }
            }

            if flip_vertical {
                // Flip vertically.
                tile_row = 7 - tile_row;
            }
            let pattern_addr = 0
                | tile_row
                | ((tile as u16) << 4)
                | ((sprite_table as u16) << 12);
            let lo = s.ppu_peek(pattern_addr);
            let hi = s.ppu_peek(pattern_addr + 8);

            // Don't draw non-existent sprites.
            if y_pos == 0xFF {
                return;
            }

            for i in 0..8 {
                let x_off = if attribute & 0x40 == 0 {
                    7 - i
                } else {
                    // Flipped horizontally.
                    i
                };
                let buf_x = (x_pos + x_off) as usize;
                let mut entry = &mut s.ppu.sprite_buffer[buf_x];
                if entry.id == 0xFF || (entry.color & 0xF) == 0 {
                    // No sprite is here yet, so put this one.
                    entry.id = n as u8;
                    entry.color = 0b10000
                        | (((lo & (1 << i)) > 0) as u8) << 0
                        | (((hi & (1 << i)) > 0) as u8) << 1
                        | (attribute & 0b11) << 2;
                    entry.priority = (attribute & 0b00100000) == 0;
                    entry.sprite0 = s.ppu.sprite_eval_has_sprite0 && (n == 0);
                }
            }

            // Add to buffer.
        }
        _ => {}
    }
}

fn increment_scroll_y(ppu: &mut PpuState) {
    if ppu.v & 0x7000 != 0x7000 {
        ppu.v += 0x1000;
    } else {
        ppu.v &= 0x8FFF;
        let mut y = (ppu.v & 0x03E0) >> 5;
        if y == 29 {
            y = 0;
            ppu.v ^= 0x0800;
        } else if y == 31 {
            y = 0;
        } else {
            y += 1;
        }
        ppu.v = (ppu.v & 0xFC1F) | (y << 5)
    }
}

fn increment_scroll_x(ppu: &mut PpuState) {
    if ppu.v & 0x001F == 31 {
        ppu.v &= 0xFFE0;
        ppu.v ^= 0x0400;
    } else {
        ppu.v += 1;
    }
}

fn render_pixel(s: &mut State) {
    let x = (s.ppu.tick - 1) as usize;
    let y = s.ppu.scanline as usize;
    let bg_index = (s.ppu.bg_data_index + (s.ppu.x as usize)) % 24;
    let mut bg_pixel = s.ppu.bg_data[bg_index];
    let mut sprite_pixel = s.ppu.sprite_buffer[x].color;

    /*if y < 128 {
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
    }*/
    if x < 8 {
        if !s.ppu.flag_show_background_left {
            bg_pixel = 0;
        }
        if !s.ppu.flag_show_sprites_left {
            sprite_pixel = 0;
        }
    }

    let bg_visible = (bg_pixel & 0x3) != 0;
    let sprite_visible = (sprite_pixel & 0x3) != 0;
    let col = if !bg_visible && !sprite_visible {
        0
    } else if !bg_visible {
        sprite_pixel
    } else if !sprite_visible {
        bg_pixel
    } else {
        if s.ppu.sprite_buffer[x].sprite0 {
            s.ppu.sprite0_hit = true;
        }
        if s.ppu.sprite_buffer[x].priority { sprite_pixel } else { bg_pixel }
    };

    let pixel = COLORS[s.ppu.palette[(col & 0x1F) as usize] as usize];
    let frame = &mut s.ppu.frame_buffer;
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
    let mut pattern_hi = s.ppu_peek(pattern_addr | 0x8) as u16;

    for i in 0..8 {
        let pixel_data = at_data | (pattern_lo & 0x1) | ((pattern_hi & 0x1) << 1);
        pattern_lo >>= 1;
        pattern_hi >>= 1;
        let ind = (s.ppu.bg_data_index + 24 - 1 - i) % s.ppu.bg_data.len();
        s.ppu.bg_data[ind] = pixel_data as u8;
    }
}

pub fn peek_register(s: &mut State, register: u16) -> u8 {
    s.ppu.latch = match register {
        2 => {
            // PPUSTATUS
            let data = (s.ppu.latch & 0x1F)
                | (s.ppu.sprite_overflow) << 5
                | (s.ppu.sprite0_hit as u8) << 6
                | (s.ppu.vblank) << 7;

            s.ppu.vblank = 0;
            s.ppu.w = 0;
            data
        }
        4 => {
            // OAMDATA
            // TODO: handle returning (0xFF [or others]) during rendering
            s.ppu.oam_1[s.ppu.oam_addr]
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
            // OAMADDR
            s.ppu.oam_addr = data as usize;
        }
        4 => {
            // OAMDATA
            if !s.ppu.is_rendering {
                s.ppu.oam_1[s.ppu.oam_addr] = data;
                s.ppu.oam_addr = (s.ppu.oam_addr + 1) & 0xFF;
            }
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
        0x4014 => {
            // OAMDMA
            let cpu_cycles = s.cpu.cycles;
            let addr = (data as u16) << 8;
            for i in 0..256 {
                let data = s.cpu_peek(addr | (i as u16));
                s.ppu.oam_1[s.ppu.oam_addr] = data;
                s.ppu.oam_addr = (s.ppu.oam_addr + 1) & 0xFF;
            }
            s.cpu.cycles = cpu_cycles + 513 + (cpu_cycles & 0x1);
        }
        _ => {}
    };
}