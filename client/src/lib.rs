use js_sys::Function;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct State {
    grej: u32,
    callback: Option<Function>,
}

#[wasm_bindgen]
impl State {
    pub fn new(i: u32) -> Self {
        State {
            grej: i,
            callback: None,
        }
    }

    pub fn grej(&self) -> u32 {
        self.grej
    }

    pub fn register(&mut self, callback: Function) {
        log::info!("hejsan");
        self.callback = Some(callback);
    }

    #[allow(unused_must_use)]
    pub fn call_it(&self) {
        if let Some(f) = &self.callback {
            f.call1(&JsValue::NULL, &JsValue::from_f64(self.grej as f64));
        }
    }
}

#[wasm_bindgen]
pub fn init_rust() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug)
        .expect("console logger failed to init");
}
