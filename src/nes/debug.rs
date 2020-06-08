use super::nes;
use sdl2::pixels::Color;
use sdl2::rect::{Point};

#[derive(Debug)]
pub enum Overlay {
    None,
    PatternTable,
}

const OVERLAYS: [Overlay; 2] = [Overlay::None, Overlay::PatternTable];

#[derive(Default)]
pub struct Debug {
    pub cpu_log: bool,
    pub overlay: usize,
}


impl Debug {
    pub fn toggle_overlay(&mut self) {
        self.overlay = (self.overlay + 1) % OVERLAYS.len();

        println!("[debug] Overlay: {:?}", OVERLAYS[self.overlay]);
    }
}

pub fn render_overlay(s: &mut nes::State, canvas: &mut sdl2::render::SurfaceCanvas) -> Result<(), String> {
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.clear();
    match OVERLAYS[s.debug.overlay] {
        Overlay::PatternTable => {
            // XXX: ppu_peek can mutate mapper state!
            for x in 0..256 {
                for y in 0..128 {
                    let mut addr = 0
                        | (y % 8)
                        | (x % 128 / 8) << 4
                        | (y % 128 / 8) << 8;
                    if x >= 128 {
                        addr |= 0x1000;
                    }
                    let lo = s.ppu_peek(addr as u16);
                    let hi = s.ppu_peek((addr + 8) as u16);
                    let col = (((lo << (x % 8) as u8) & 0x80) >> 7) | (((hi << (x % 8) as u8) & 0x80) >> 6);
                    let col = col * 60 + 30;
                    canvas.set_draw_color(Color::RGB(col, col, col));
                    canvas.draw_point(Point::new(x, y))?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}