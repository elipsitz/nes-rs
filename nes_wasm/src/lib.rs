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
}
