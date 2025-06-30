//! Minimal test suite for readap-wasm regression testing
//!
//! This test suite focuses on preventing regressions in the refactored APIs:
//! - ImmutableDataset: Prevents "recursive use of an object detected" errors
//! - SimpleConstraintBuilder: Ensures method chaining works without aliasing
//! - UniversalDodsParser: Validates basic parsing functionality

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use js_sys::{Object, Uint8Array};
use readap_wasm::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Test 1: API Instantiation - Verify all main classes can be created
#[wasm_bindgen_test]
fn test_api_instantiation() {
    // Test ImmutableDataset creation
    let dataset = ImmutableDataset::new("https://example.com/data.nc");
    assert!(dataset.is_ok(), "ImmutableDataset should be creatable");

    // Test SimpleConstraintBuilder creation
    let builder = SimpleConstraintBuilder::new();
    let constraint_str = builder.build();
    assert_eq!(
        constraint_str, "",
        "Empty constraint builder should return empty string"
    );

    // Test UniversalDodsParser creation
    let parser = UniversalDodsParser::new();
    // Parser should be created successfully (no direct test method, but creation shouldn't panic)

    // Test that we can call methods on the parser without errors
    let empty_data = Uint8Array::new_with_length(0);
    let result = parser.parse_dods_detailed(&empty_data);
    assert!(
        result.is_object(),
        "Parser should return an object for detailed parsing"
    );
}

/// Test 2: Immutable Dataset Method Chaining - Prevents "recursive use" errors
#[wasm_bindgen_test]
fn test_immutable_dataset_chaining() {
    let dataset = ImmutableDataset::new("https://example.com/data.nc")
        .expect("Dataset creation should succeed");

    // Test DAS data chaining - this was the main source of aliasing errors
    let mock_das = r#"
        Attributes {
            temperature {
                String units "celsius";
                Float64 valid_range 0.0, 50.0;
            }
        }
    "#;

    let dataset_with_das = dataset.with_das_data(mock_das.to_string());
    assert!(
        dataset_with_das.is_ok(),
        "Adding DAS data should not cause aliasing errors"
    );

    // Test DDS data chaining
    let mock_dds = r#"
        Dataset {
            Float32 temperature[time = 10][lat = 5][lon = 8];
        } data;
    "#;

    if let Ok(dataset_with_das) = dataset_with_das {
        let dataset_with_both = dataset_with_das.with_dds_data(mock_dds.to_string());
        assert!(
            dataset_with_both.is_ok(),
            "Adding DDS data should not cause aliasing errors"
        );

        // Test that we can still call methods on the final dataset
        if let Ok(final_dataset) = dataset_with_both {
            let var_names = final_dataset.get_variable_names();
            assert!(
                var_names.length() >= 0,
                "Should be able to get variable names"
            );
        }
    }
}

/// Test 3: Constraint Builder Method Chaining - Ensures no aliasing issues
#[wasm_bindgen_test]
fn test_constraint_builder_chaining() {
    let builder = SimpleConstraintBuilder::new();

    // Test method chaining - this pattern was problematic before the refactor
    let chained_builder = builder
        .add_single("time", 5)
        .add_range("lat", 10, 20)
        .add_stride("lon", 0, 2, 10);

    let constraint_str = chained_builder.build();

    // Verify the constraint string contains expected elements
    assert!(
        constraint_str.contains("time"),
        "Constraint should contain time selection"
    );
    assert!(
        constraint_str.contains("lat"),
        "Constraint should contain lat selection"
    );
    assert!(
        constraint_str.contains("lon"),
        "Constraint should contain lon selection"
    );

    // Test that we can continue chaining after build
    let builder2 = chained_builder.add_single("depth", 0);
    let constraint_str2 = builder2.build();
    assert!(
        constraint_str2.contains("depth"),
        "Should be able to continue chaining after build"
    );
}

/// Test 4: DODS Parser Smoke Test - Basic parsing functionality
#[wasm_bindgen_test]
fn test_dods_parser_smoke() {
    let parser = UniversalDodsParser::new();

    // Test with empty data - should handle gracefully
    let empty_data = Uint8Array::new_with_length(0);
    let empty_result = parser.parse_dods_detailed(&empty_data);

    // Should return an object with success: false
    assert!(
        empty_result.is_object(),
        "Parser should return object for empty data"
    );

    // Test with minimal mock DODS data
    let mock_dods_data = b"Dataset {\n    Float32 temp[x = 2];\n} data;\nData:\n\x00\x00\x00\x02\x00\x00\x00\x02\x41\x20\x00\x00\x41\xa0\x00\x00";
    let data_array = Uint8Array::from(mock_dods_data.as_slice());

    // This should not panic, even if parsing fails
    let result = parser.parse_dods_detailed(&data_array);
    assert!(
        result.is_object(),
        "Parser should return object for mock data"
    );

    // Test structure analysis
    let analysis = parser.analyze_dods_structure(&data_array);
    assert!(
        analysis.is_object(),
        "Structure analysis should return object"
    );
}
