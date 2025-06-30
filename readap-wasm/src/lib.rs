mod coordinate_resolver;
mod dataset;
mod url_builder;
mod utils;

use wasm_bindgen::prelude::*;

// Re-export main API components
pub use coordinate_resolver::*;
pub use dataset::*;
pub use url_builder::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}
