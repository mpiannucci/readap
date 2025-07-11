use wasm_bindgen_test::*;
use readap_wasm::parse_das_attributes_js;
use js_sys::Reflect;

#[wasm_bindgen_test]
fn test_parse_das_attributes() {
    let das_input = r#"Attributes {
    time {
        String long_name "Epoch Time";
        String short_name "time";
        String standard_name "time";
        String units "seconds since 1970-01-01 00:00:00 UTC";
    }
    frequency {
        String long_name "Frequency";
        String short_name "frequency";
        String standard_name "frequency";
        String units "Hz";
    }
    spectral_wave_density {
        String long_name "Spectral Wave Density";
        String short_name "swden";
        String standard_name "spectral_wave_density";
        String units "(meter * meter)/Hz";
        Float32 _FillValue 999.0;
    }
}"#;

    let result = parse_das_attributes_js(das_input);
    assert!(result.is_ok());
    
    let js_obj = result.unwrap();
    assert!(js_obj.is_object());
    
    // Check that we have the expected variables
    let time_var = Reflect::get(&js_obj, &"time".into()).unwrap();
    assert!(time_var.is_object());
    
    let freq_var = Reflect::get(&js_obj, &"frequency".into()).unwrap();
    assert!(freq_var.is_object());
    
    let wave_var = Reflect::get(&js_obj, &"spectral_wave_density".into()).unwrap();
    assert!(wave_var.is_object());
    
    // Check an attribute in detail
    let long_name_attr = Reflect::get(&time_var, &"long_name".into()).unwrap();
    assert!(long_name_attr.is_object());
    
    let data_type = Reflect::get(&long_name_attr, &"dataType".into()).unwrap();
    assert_eq!(data_type.as_string().unwrap(), "String");
    
    let name = Reflect::get(&long_name_attr, &"name".into()).unwrap();
    assert_eq!(name.as_string().unwrap(), "long_name");
    
    let value = Reflect::get(&long_name_attr, &"value".into()).unwrap();
    assert_eq!(value.as_string().unwrap(), "Epoch Time");
    
    // Check a float value
    let fill_value_attr = Reflect::get(&wave_var, &"_FillValue".into()).unwrap();
    let fill_data_type = Reflect::get(&fill_value_attr, &"dataType".into()).unwrap();
    assert_eq!(fill_data_type.as_string().unwrap(), "Float32");
    
    let fill_value = Reflect::get(&fill_value_attr, &"value".into()).unwrap();
    assert_eq!(fill_value.as_f64().unwrap(), 999.0);
}

#[wasm_bindgen_test]
fn test_parse_invalid_das() {
    let invalid_input = "Not a valid DAS format";
    let result = parse_das_attributes_js(invalid_input);
    assert!(result.is_err());
}