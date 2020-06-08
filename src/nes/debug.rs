use super::nes;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

#[derive(Debug)]
pub enum Overlay {
    None,
}

const OVERLAYS: [Overlay; 1] = [Overlay::None];

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

pub fn render_overlay(state: &nes::State, canvas: &mut sdl2::render::SurfaceCanvas) -> Result<(), String> {
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.clear();
    match OVERLAYS[state.debug.overlay] {
        _ => {}
    }
    Ok(())
}