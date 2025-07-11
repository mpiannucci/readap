use wasm_bindgen_test::*;
use readap_wasm::{parse_dds_dataset_js, DdsDatasetWrapper, DdsArrayWrapper, DdsGridWrapper};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse_simple_dds_dataset() {
    let dds_content = r#"Dataset {
    Int32 time[time = 7];
    Float32 frequency[frequency = 64];
} test_dataset;"#;

    let result = parse_dds_dataset_js(dds_content);
    assert!(result.is_ok());
    
    let dataset = result.unwrap();
    assert!(!dataset.is_undefined());
    assert!(!dataset.is_null());
}

#[wasm_bindgen_test]
fn test_parse_grid_dds_dataset() {
    let dds_content = r#"Dataset {
    Int32 time[time = 7];
    Float32 frequency[frequency = 64];
    Grid {
     ARRAY:
        Float32 spectral_wave_density[time = 7][frequency = 64][latitude = 1][longitude = 1];
     MAPS:
        Int32 time[time = 7];
        Float32 frequency[frequency = 64];
        Float32 latitude[latitude = 1];
        Float32 longitude[longitude = 1];
    } spectral_wave_density;
} data/swden/44097/44097w9999.nc;"#;

    let result = parse_dds_dataset_js(dds_content);
    assert!(result.is_ok());
    
    let dataset = result.unwrap();
    assert!(!dataset.is_undefined());
    assert!(!dataset.is_null());
}

#[wasm_bindgen_test]
fn test_dds_dataset_wrapper() {
    let dds_content = r#"Dataset {
    Int32 time[time = 7];
    Float32 frequency[frequency = 64];
    Grid {
     ARRAY:
        Float32 temperature[time = 7][frequency = 64];
     MAPS:
        Int32 time[time = 7];
        Float32 frequency[frequency = 64];
    } temperature;
} test_dataset;"#;

    let dataset = DdsDatasetWrapper::new(dds_content);
    assert!(dataset.is_ok());
    
    let dataset = dataset.unwrap();
    assert_eq!(dataset.name(), "test_dataset");
    assert_eq!(dataset.get_variable_count(), 3);
    
    let variables = dataset.list_variables();
    assert_eq!(variables.len(), 3);
    assert!(variables.contains(&"time".to_string()));
    assert!(variables.contains(&"frequency".to_string()));
    assert!(variables.contains(&"temperature".to_string()));
    
    let coordinates = dataset.list_coordinates();
    assert_eq!(coordinates.len(), 2);
    assert!(coordinates.contains(&"time".to_string()));
    assert!(coordinates.contains(&"frequency".to_string()));
    
    assert!(dataset.has_variable("temperature"));
    assert!(!dataset.has_variable("nonexistent"));
    
    assert!(dataset.has_coordinate("time"));
    assert!(!dataset.has_coordinate("nonexistent"));
}

#[wasm_bindgen_test]
fn test_dds_dataset_wrapper_metadata() {
    let dds_content = r#"Dataset {
    Float32 latitude[latitude = 5];
    Float32 longitude[longitude = 10];
    Int32 time[time = 100];
    Grid {
     ARRAY:
        Float32 temperature[time = 100][latitude = 5][longitude = 10];
     MAPS:
        Int32 time[time = 100];
        Float32 latitude[latitude = 5];
        Float32 longitude[longitude = 10];
    } temperature;
} test_dataset;"#;

    let dataset = DdsDatasetWrapper::new(dds_content).unwrap();
    
    // Test variable info
    let temp_info = dataset.get_variable_info("temperature");
    assert!(!temp_info.is_null());
    
    let lat_info = dataset.get_variable_info("latitude");
    assert!(!lat_info.is_null());
    
    let nonexistent_info = dataset.get_variable_info("nonexistent");
    assert!(nonexistent_info.is_null());
    
    // Test coordinate info
    let time_coord_info = dataset.get_coordinate_info("time");
    assert!(!time_coord_info.is_null());
    
    let nonexistent_coord_info = dataset.get_coordinate_info("nonexistent");
    assert!(nonexistent_coord_info.is_null());
    
    // Test get variable
    let temp_var = dataset.get_variable("temperature");
    assert!(!temp_var.is_null());
    
    let lat_var = dataset.get_variable("latitude");
    assert!(!lat_var.is_null());
    
    let nonexistent_var = dataset.get_variable("nonexistent");
    assert!(nonexistent_var.is_null());
    
    // Test get variable at index
    let first_var = dataset.get_variable_at(0);
    assert!(!first_var.is_null());
    
    let out_of_bounds = dataset.get_variable_at(100);
    assert!(out_of_bounds.is_null());
}

#[wasm_bindgen_test]
fn test_dds_array_wrapper() {
    let array_content = "Int32 time[time = 7];";
    
    let array = DdsArrayWrapper::new(array_content);
    assert!(array.is_ok());
    
    let array = array.unwrap();
    assert_eq!(array.name(), "time");
    assert_eq!(array.data_type(), "Int32");
    assert_eq!(array.array_length(), 7);
    assert_eq!(array.byte_count(), 8 + 7 * 4); // 8 bytes header + 7 * 4 bytes data
    
    let coordinates = array.get_coordinates();
    assert!(!coordinates.is_undefined());
    assert!(!coordinates.is_null());
    
    let js_obj = array.to_js();
    assert!(js_obj.is_ok());
}

#[wasm_bindgen_test]
fn test_dds_array_wrapper_multi_dimensional() {
    let array_content = "Float32 data[time = 7][frequency = 64][latitude = 1][longitude = 1];";
    
    let array = DdsArrayWrapper::new(array_content);
    assert!(array.is_ok());
    
    let array = array.unwrap();
    assert_eq!(array.name(), "data");
    assert_eq!(array.data_type(), "Float32");
    assert_eq!(array.array_length(), 7 * 64 * 1 * 1);
    assert_eq!(array.byte_count(), 8 + 7 * 64 * 4); // 8 bytes header + data bytes
    
    let coordinates = array.get_coordinates();
    assert!(!coordinates.is_undefined());
    assert!(!coordinates.is_null());
}

#[wasm_bindgen_test]
fn test_dds_grid_wrapper() {
    let grid_content = r#"Grid {
     ARRAY:
        Float32 temperature[time = 7][latitude = 5];
     MAPS:
        Int32 time[time = 7];
        Float32 latitude[latitude = 5];
    } temperature;"#;
    
    let grid = DdsGridWrapper::new(grid_content);
    assert!(grid.is_ok());
    
    let grid = grid.unwrap();
    assert_eq!(grid.name(), "temperature");
    assert!(grid.byte_count() > 0);
    assert!(grid.coords_offset() > 0);
    
    let coord_offsets = grid.get_coord_offsets();
    assert_eq!(coord_offsets.len(), 2);
    
    let array = grid.get_array();
    assert!(array.is_ok());
    
    let coordinates = grid.get_coordinates();
    assert!(coordinates.is_ok());
    
    let js_obj = grid.to_js();
    assert!(js_obj.is_ok());
}

#[wasm_bindgen_test]
fn test_parse_invalid_dds() {
    let invalid_dds = "Invalid DDS content";
    
    let result = parse_dds_dataset_js(invalid_dds);
    assert!(result.is_err());
    
    let dataset_result = DdsDatasetWrapper::new(invalid_dds);
    assert!(dataset_result.is_err());
    
    let array_result = DdsArrayWrapper::new(invalid_dds);
    assert!(array_result.is_err());
    
    let grid_result = DdsGridWrapper::new(invalid_dds);
    assert!(grid_result.is_err());
}

#[wasm_bindgen_test]
fn test_parse_structure_and_sequence() {
    let dds_content = r#"Dataset {
    Structure {
        Int32 id;
        Float32 value;
    } measurement;
    Sequence {
        Int32 timestamp;
        Float32 temperature;
    } readings;
} test_dataset;"#;

    let result = parse_dds_dataset_js(dds_content);
    assert!(result.is_ok());
    
    let dataset = DdsDatasetWrapper::new(dds_content);
    assert!(dataset.is_ok());
    
    let dataset = dataset.unwrap();
    assert_eq!(dataset.name(), "test_dataset");
    assert_eq!(dataset.get_variable_count(), 2);
    
    let variables = dataset.list_variables();
    assert!(variables.contains(&"measurement".to_string()));
    assert!(variables.contains(&"readings".to_string()));
    
    let measurement_var = dataset.get_variable("measurement");
    assert!(!measurement_var.is_null());
    
    let readings_var = dataset.get_variable("readings");
    assert!(!readings_var.is_null());
}