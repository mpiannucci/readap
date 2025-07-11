use readap::das::parse_das_attributes;
use wasm_bindgen::prelude::*;
use crate::converters::das_attributes_to_js_object;

#[wasm_bindgen(js_name = parseDasAttributes)]
pub fn parse_das_attributes_js(input: &str) -> Result<JsValue, String> {
    match parse_das_attributes(input) {
        Ok(attributes) => das_attributes_to_js_object(&attributes)
            .map_err(|e| format!("Error converting to JavaScript object: {:?}", e)),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}