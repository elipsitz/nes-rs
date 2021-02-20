use super::nes;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Overlay {
    None,
    PatternTable,
    Sprites,
}

const OVERLAYS: [Overlay; 3] = [Overlay::None, Overlay::PatternTable, Overlay::Sprites];

pub struct Debug {
    pub cpu_log: bool,
    pub overlay: usize,
    pub overlay_buffer: [u8; nes::FRAME_SIZE],
}

impl Default for Debug {
    fn default() -> Self {
        Debug {
            cpu_log: false,
            overlay: 0,
            overlay_buffer: [0; nes::FRAME_SIZE],
        }
    }
}

impl Debug {
    pub fn toggle_overlay(&mut self) {
        self.overlay = (self.overlay + 1) % OVERLAYS.len();

        println!("[debug] Overlay: {:?}", OVERLAYS[self.overlay]);
    }
}

#[derive(Copy, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn gray(x: u8) -> Color {
        Color {
            r: x,
            g: x,
            b: x,
            a: 255,
        }
    }

    fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }
}

struct DebugCanvas<'a>(&'a mut [u8; nes::FRAME_SIZE]);

impl DebugCanvas<'_> {
    pub fn clear(&mut self) {
        self.0.iter_mut().for_each(|p| *p = 0);
    }

    pub fn draw_point(&mut self, x: usize, y: usize, color: Color) {
        if x < nes::FRAME_WIDTH && y <= nes::FRAME_HEIGHT {
            let c = (x + y * nes::FRAME_WIDTH) * nes::FRAME_DEPTH;
            self.0[c + 0] = color.r;
            self.0[c + 1] = color.g;
            self.0[c + 2] = color.b;
            self.0[c + 3] = color.a;
        }
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        for i in x..(x + w) {
            self.draw_point(i, y, color);
            self.draw_point(i, y + h - 1, color);
        }
        for j in y..(y + h) {
            self.draw_point(x, j, color);
            self.draw_point(x + w - 1, j, color);
        }
    }
}

pub fn update_overlay(s: &mut nes::State) {
    let overlay = OVERLAYS[s.debug.overlay];
    if overlay == Overlay::None {
        return;
    }

    let mut temp_buffer = [0u8; nes::FRAME_SIZE];
    let mut canvas = DebugCanvas(&mut temp_buffer);
    canvas.clear();

    match overlay {
        Overlay::PatternTable => {
            // XXX: ppu_peek can mutate mapper state!
            for x in 0..256 {
                for y in 0..128 {
                    let mut addr = 0 | (y % 8) | (x % 128 / 8) << 4 | (y % 128 / 8) << 8;
                    if x >= 128 {
                        addr |= 0x1000;
                    }
                    let lo = s.ppu_peek(addr as u16);
                    let hi = s.ppu_peek((addr + 8) as u16);
                    let col = (((lo << (x % 8) as u8) & 0x80) >> 7)
                        | (((hi << (x % 8) as u8) & 0x80) >> 6);
                    let col = col * 60 + 30;
                    canvas.draw_point(x, y, Color::gray(col));
                }
            }
        }
        Overlay::Sprites => {
            for i in (0..256).step_by(4) {
                let x = s.ppu.oam_1[i + 3] as i32;
                let y = (s.ppu.oam_1[i + 0] as i32) + 1;
                let attr = s.ppu.oam_1[i + 2];
                let priority = (attr & 0b00100000) > 0;
                let color = if priority {
                    // Magenta: behind background
                    Color::rgb(255, 0, 255)
                } else {
                    // Lime: in front of background
                    Color::rgb(0, 255, 0)
                };
                let height = 8 << s.ppu.flag_sprite_size;
                canvas.draw_rect(x as usize, y as usize, 8, height, color);
            }
        }
        _ => {}
    }

    s.debug.overlay_buffer.copy_from_slice(&temp_buffer);
}
