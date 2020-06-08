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