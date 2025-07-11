use readap::dds::DdsDataset;
use wasm_bindgen::prelude::*;
use crate::converters::dds_dataset_to_js_object;

#[wasm_bindgen(js_name = parseDdsDataset)]
pub fn parse_dds_dataset_js(input: &str) -> Result<JsValue, String> {
    match DdsDataset::from_bytes(input) {
        Ok(dataset) => dds_dataset_to_js_object(&dataset)
            .map_err(|e| format!("Error converting to JavaScript object: {:?}", e)),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}