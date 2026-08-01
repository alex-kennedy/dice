use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn on_start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn get_greeting() -> String {
    "Hello from dice-wasm!".to_string()
}
