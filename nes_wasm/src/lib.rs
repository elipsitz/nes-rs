use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Emulator {
    nes: nes_core::Nes,
}

#[wasm_bindgen]
impl Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(rom: &[u8]) -> Emulator {
        let cartridge = nes_core::Cartridge::load(rom);
        let debug = nes_core::Debug::default();
        let nes = nes_core::Nes::new(debug, cartridge);
        Emulator { nes }
    }

    pub fn emulate_frame(&mut self) {
        self.nes.emulate_frame();
    }

    pub fn get_frame_buffer(&self, out: &mut [u8]) {
        out.copy_from_slice(self.nes.get_frame_buffer());
    }

    pub fn set_controller1_state(
        &mut self,
        a: bool,
        b: bool,
        select: bool,
        start: bool,
        left: bool,
        right: bool,
        up: bool,
        down: bool,
    ) {
        let mut state = nes_core::ControllerState::default();
        state.a = a;
        state.b = b;
        state.select = select;
        state.start = start;
        state.left = left;
        state.right = right;
        state.up = up;
        state.down = down;
        self.nes.set_controller1_state(state);
    }
}
