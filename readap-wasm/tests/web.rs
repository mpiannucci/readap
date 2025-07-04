//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;
use readap_wasm::{parse_dds, parse_das, JsUrlBuilder};
use js_sys::{Array, Object, Reflect};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse_dds() {
    let dds_content = r#"Dataset {
    Float32 temp[time = 10][lat = 180][lon = 360];
    Float64 time[time = 10];
    Float32 lat[lat = 180];
    Float32 lon[lon = 360];
} example.nc;"#;

    let result = parse_dds(dds_content).expect("Failed to parse DDS");
    
    // Verify it's an object
    assert!(result.is_object());
    
    // Check name
    let name = Reflect::get(&result, &"name".into()).unwrap();
    assert_eq!(name.as_string().unwrap(), "example.nc");
    
    // Check variables array exists
    let vars = Reflect::get(&result, &"variables".into()).unwrap();
    assert!(vars.is_object());
    
    // Check coordinate variables array exists  
    let coords = Reflect::get(&result, &"coordinate_variables".into()).unwrap();
    assert!(coords.is_object());
}

#[wasm_bindgen_test]
fn test_parse_das() {
    let das_content = r#"Attributes {
    temp {
        String long_name "Temperature";
        String units "celsius";
        Float32 _FillValue -999.0;
    }
    time {
        String units "days since 1970-01-01";
    }
}"#;

    let result = parse_das(das_content).expect("Failed to parse DAS");
    
    // Verify it's an object
    assert!(result.is_object());
    
    // Check that temp attributes exist
    let temp_attrs = Reflect::get(&result, &"temp".into()).unwrap();
    assert!(temp_attrs.is_object());
}

#[wasm_bindgen_test]
fn test_url_builder() {
    let mut builder = JsUrlBuilder::new("https://example.com/data.nc").expect("Failed to create URL builder");
    
    // Test basic URL generation
    assert_eq!(builder.das_url(), "https://example.com/data.nc.das");
    assert_eq!(builder.dds_url(), "https://example.com/data.nc.dds");
    assert_eq!(builder.dods_url(), "https://example.com/data.nc.dods");
    
    // Test adding variables
    builder.add_variable("temp").unwrap();
    builder.add_variable("time").unwrap();
    
    // Test adding constraints
    builder.add_constraint("lat", "40.0").unwrap();
    builder.add_range("time", 0, 10).unwrap();
    
    // Verify URLs contain query parameters
    let dods_url = builder.dods_url();
    assert!(dods_url.contains("?"));
    assert!(dods_url.contains("temp"));
}

#[wasm_bindgen_test]
fn test_url_builder_with_array() {
    let mut builder = JsUrlBuilder::new("https://example.com/data.nc").expect("Failed to create URL builder");
    
    // Create JavaScript array of variables
    let vars = Array::new();
    vars.push(&JsValue::from_str("temp"));
    vars.push(&JsValue::from_str("pressure"));
    vars.push(&JsValue::from_str("humidity"));
    
    // Add multiple variables at once
    builder.add_variables(&vars).unwrap();
    
    let dods_url = builder.dods_url();
    assert!(dods_url.contains("temp"));
    assert!(dods_url.contains("pressure"));
    assert!(dods_url.contains("humidity"));
}

#[wasm_bindgen_test]
fn test_parse_error_handling() {
    let invalid_dds = "This is not valid DDS content";
    
    let result = parse_dds(invalid_dds);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.as_string().unwrap().contains("Parse error"));
}

#[wasm_bindgen_test]
fn test_url_builder_clone() {
    let mut builder1 = JsUrlBuilder::new("https://example.com/data.nc").expect("Failed to create URL builder");
    builder1.add_variable("temp").unwrap();
    
    let builder2 = builder1.clone_builder();
    
    // Both should produce the same URL
    assert_eq!(builder1.dods_url(), builder2.dods_url());
}