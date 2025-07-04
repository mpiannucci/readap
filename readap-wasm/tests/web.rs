//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_bindgen::JsValue;
use readap_wasm::{parse_dds, parse_das, UrlBuilder};
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
    
    // Check values array exists
    let values = Reflect::get(&result, &"values".into()).unwrap();
    assert!(values.is_object());
    
    // Check variables list exists  
    let variables = Reflect::get(&result, &"variables".into()).unwrap();
    assert!(variables.is_object());
    
    // Check coordinates list exists
    let coordinates = Reflect::get(&result, &"coordinates".into()).unwrap();
    assert!(coordinates.is_object());
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
    
    // Check that time attributes exist
    let time_attrs = Reflect::get(&result, &"time".into()).unwrap();
    assert!(time_attrs.is_object());
}

#[wasm_bindgen_test]
fn test_url_builder() {
    let builder = UrlBuilder::new("https://example.com/data.nc");
    
    // Test basic URL generation
    assert_eq!(builder.das_url(), "https://example.com/data.nc.das");
    assert_eq!(builder.dds_url(), "https://example.com/data.nc.dds");
    
    let dods_url = builder.dods_url().expect("Failed to generate DODS URL");
    assert_eq!(dods_url, "https://example.com/data.nc.dods");
}

#[wasm_bindgen_test]
fn test_url_builder_with_variables() {
    let builder = UrlBuilder::new("https://example.com/data.nc")
        .add_variable("temp")
        .add_variable("time");
    
    let dods_url = builder.dods_url().expect("Failed to generate DODS URL");
    assert!(dods_url.contains("?"));
    assert!(dods_url.contains("temp"));
    assert!(dods_url.contains("time"));
}

#[wasm_bindgen_test]
fn test_url_builder_with_array() {
    let builder = UrlBuilder::new("https://example.com/data.nc");
    
    // Create JavaScript array of variables
    let vars = Array::new();
    vars.push(&JsValue::from_str("temp"));
    vars.push(&JsValue::from_str("pressure"));
    vars.push(&JsValue::from_str("humidity"));
    
    // Add multiple variables at once
    let builder = builder.add_variables(&vars);
    
    let dods_url = builder.dods_url().expect("Failed to generate DODS URL");
    assert!(dods_url.contains("temp"));
    assert!(dods_url.contains("pressure"));
    assert!(dods_url.contains("humidity"));
}

#[wasm_bindgen_test]
fn test_url_builder_with_simple_constraints() {
    let builder = UrlBuilder::new("https://example.com/data.nc")
        .add_variable("temp")
        .add_single_index("temp", 5);
    
    let dods_url = builder.dods_url().expect("Failed to generate DODS URL");
    assert!(dods_url.contains("temp[5]"));
}

#[wasm_bindgen_test]
fn test_url_builder_with_ranges() {
    let builder = UrlBuilder::new("https://example.com/data.nc")
        .add_variable("temp")
        .add_range("temp", 0, 10, None);
    
    let dods_url = builder.dods_url().expect("Failed to generate DODS URL");
    assert!(dods_url.contains("temp[0:10]"));
}

#[wasm_bindgen_test]
fn test_url_builder_with_stride() {
    let builder = UrlBuilder::new("https://example.com/data.nc")
        .add_variable("temp")
        .add_range("temp", 0, 20, Some(2));
    
    let dods_url = builder.dods_url().expect("Failed to generate DODS URL");
    assert!(dods_url.contains("temp[0:2:20]"));
}

#[wasm_bindgen_test]
fn test_url_builder_multidimensional() {
    let builder = UrlBuilder::new("https://example.com/data.nc")
        .add_variable("temp");
    
    // Create constraint array with mixed types
    let constraints = Array::new();
    constraints.push(&JsValue::from_f64(5.0)); // Single index
    
    // Range object
    let range_obj = Object::new();
    Reflect::set(&range_obj, &"start".into(), &JsValue::from_f64(0.0)).unwrap();
    Reflect::set(&range_obj, &"end".into(), &JsValue::from_f64(10.0)).unwrap();
    constraints.push(&range_obj);
    
    let builder = builder.add_multidimensional_constraint("temp", &constraints)
        .expect("Failed to add multidimensional constraint");
    
    let dods_url = builder.dods_url().expect("Failed to generate DODS URL");
    assert!(dods_url.contains("temp"));
    assert!(dods_url.contains("["));
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
    let builder1 = UrlBuilder::new("https://example.com/data.nc")
        .add_variable("temp");
    
    let builder2 = builder1.clone_builder();
    
    // Both should produce the same URL
    let url1 = builder1.dods_url().unwrap();
    let url2 = builder2.dods_url().unwrap();
    assert_eq!(url1, url2);
}

#[wasm_bindgen_test]
fn test_url_builder_clear_operations() {
    let builder = UrlBuilder::new("https://example.com/data.nc")
        .add_variable("temp")
        .add_variable("pressure")
        .add_range("temp", 0, 10, None);
    
    // Clear variables but keep constraints
    let builder = builder.clear_variables();
    let url = builder.dods_url().unwrap();
    assert!(url.contains("temp[0:10]")); // Constraint should remain
    assert!(!url.contains("pressure")); // Variable should be gone
    
    // Clear all
    let builder = builder.clear_all();
    let url = builder.dods_url().unwrap();
    assert_eq!(url, "https://example.com/data.nc.dods");
}