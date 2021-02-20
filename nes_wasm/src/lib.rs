use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn load_rom(data: &[u8]) {
    alert(&format!("Length: {}", data.len()));
}
