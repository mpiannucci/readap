//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use readap_wasm::{parse_das, parse_dds, UrlBuilder};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse_dds() {
    let dds_content = r#"Dataset {
    Float32 temperature[lat = 180][lon = 360];
    Float64 lat[lat = 180];
    Float64 lon[lon = 360];
} example;"#;

    let result = parse_dds(dds_content);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_parse_das() {
    let das_content = r#"Attributes {
    temperature {
        String units "degrees_C";
        String long_name "Sea Surface Temperature";
    }
}"#;

    let result = parse_das(das_content);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_url_builder() {
    let builder = UrlBuilder::new("https://example.com/data.nc");
    let url = builder
        .add_variable("temperature")
        .dods_url()
        .expect("Failed to generate URL");
    assert!(url.contains("temperature"));
    assert!(url.contains("https://example.com/data.nc"));
}

#[wasm_bindgen_test]
fn test_url_builder_basic_functionality() {
    let builder = UrlBuilder::new("https://example.com/data.nc");
    let url = builder
        .add_variable("time")
        .dods_url()
        .expect("Failed to generate URL");
    assert!(url.contains("time"));
    assert!(url.contains("https://example.com/data.nc"));
}

#[wasm_bindgen_test]
fn test_parse_error_handling() {
    let invalid_dds = "This is not valid DDS content";

    let result = parse_dds(invalid_dds);
    assert!(result.is_err());
}

// Note: Testing parse_dods would require actual binary DODS data
// which is complex to embed in tests. In practice, integration tests
// would use real OpenDAP server responses.
