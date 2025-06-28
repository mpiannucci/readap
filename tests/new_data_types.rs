use readap::{data::*, das::*, dds::*};

#[cfg(test)]
mod data_type_tests {
    use super::*;

    #[test]
    fn test_parse_all_data_types() {
        // Test parsing all supported data types
        let test_cases = vec![
            ("Byte", DataType::Byte),
            ("Int16", DataType::Int16),
            ("UInt16", DataType::UInt16),
            ("Int32", DataType::Int32),
            ("UInt32", DataType::UInt32),
            ("Float32", DataType::Float32),
            ("Float64", DataType::Float64),
            ("String", DataType::String),
            ("URL", DataType::URL),
        ];

        for (input, expected) in test_cases {
            let (_, parsed) = DataType::parse(input).unwrap();
            assert_eq!(parsed, expected, "Failed to parse {}", input);
        }
    }

    #[test]
    fn test_data_type_byte_counts() {
        assert_eq!(DataType::Byte.byte_count(), 1);
        assert_eq!(DataType::Int16.byte_count(), 2);
        assert_eq!(DataType::UInt16.byte_count(), 2);
        assert_eq!(DataType::Int32.byte_count(), 4);
        assert_eq!(DataType::UInt32.byte_count(), 4);
        assert_eq!(DataType::Float32.byte_count(), 4);
        assert_eq!(DataType::Float64.byte_count(), 8);
        assert_eq!(DataType::String.byte_count(), 0); // Variable length
        assert_eq!(DataType::URL.byte_count(), 0); // Variable length
    }

    #[test]
    fn test_data_value_conversions() {
        // Test Byte conversions
        let byte_val = DataValue::Byte(42);
        assert_eq!(TryInto::<i32>::try_into(byte_val.clone()).unwrap(), 42);
        assert_eq!(TryInto::<f32>::try_into(byte_val.clone()).unwrap(), 42.0);
        assert_eq!(TryInto::<f64>::try_into(byte_val).unwrap(), 42.0);

        // Test Int16 conversions
        let int16_val = DataValue::Int16(-1000);
        assert_eq!(TryInto::<i32>::try_into(int16_val.clone()).unwrap(), -1000);
        assert_eq!(TryInto::<f32>::try_into(int16_val.clone()).unwrap(), -1000.0);
        assert_eq!(TryInto::<f64>::try_into(int16_val).unwrap(), -1000.0);

        // Test UInt16 conversions
        let uint16_val = DataValue::UInt16(65000);
        assert_eq!(TryInto::<i32>::try_into(uint16_val.clone()).unwrap(), 65000);
        assert_eq!(TryInto::<f32>::try_into(uint16_val.clone()).unwrap(), 65000.0);
        assert_eq!(TryInto::<f64>::try_into(uint16_val).unwrap(), 65000.0);

        // Test UInt32 conversions
        let uint32_val = DataValue::UInt32(4000000000);
        assert_eq!(TryInto::<i32>::try_into(uint32_val.clone()).unwrap(), -294967296); // Overflow
        assert_eq!(TryInto::<f32>::try_into(uint32_val.clone()).unwrap(), 4000000000.0);
        assert_eq!(TryInto::<f64>::try_into(uint32_val).unwrap(), 4000000000.0);

        // Test Float64 conversions
        let float64_val = DataValue::Float64(3.14159265359);
        assert_eq!(TryInto::<i32>::try_into(float64_val.clone()).unwrap(), 3);
        assert!((TryInto::<f32>::try_into(float64_val.clone()).unwrap() - 3.1415927).abs() < 0.0001);
        assert_eq!(TryInto::<f64>::try_into(float64_val).unwrap(), 3.14159265359);

        // Test String conversions
        let string_val = DataValue::String("test".to_string());
        assert_eq!(TryInto::<String>::try_into(string_val).unwrap(), "test");

        // Test URL conversions
        let url_val = DataValue::URL("http://example.com".to_string());
        assert_eq!(TryInto::<String>::try_into(url_val).unwrap(), "http://example.com");
    }

    #[test]
    fn test_invalid_conversions() {
        let string_val = DataValue::String("test".to_string());
        assert!(TryInto::<i32>::try_into(string_val.clone()).is_err());
        assert!(TryInto::<f32>::try_into(string_val.clone()).is_err());
        assert!(TryInto::<f64>::try_into(string_val).is_err());

        let url_val = DataValue::URL("http://example.com".to_string());
        assert!(TryInto::<i32>::try_into(url_val.clone()).is_err());
        assert!(TryInto::<f32>::try_into(url_val.clone()).is_err());
        assert!(TryInto::<f64>::try_into(url_val).is_err());

        let byte_val = DataValue::Byte(42);
        assert!(TryInto::<String>::try_into(byte_val).is_err());
    }
}

#[cfg(test)]
mod das_attribute_tests {
    use super::*;

    #[test]
    fn test_parse_byte_attribute() {
        let input = "Byte quality_flag 1;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::Byte);
        assert_eq!(attr.name, "quality_flag");
        if let DataValue::Byte(val) = attr.value {
            assert_eq!(val, 1);
        } else {
            panic!("Expected Byte value");
        }
    }

    #[test]
    fn test_parse_int16_attribute() {
        let input = "Int16 elevation -500;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::Int16);
        assert_eq!(attr.name, "elevation");
        if let DataValue::Int16(val) = attr.value {
            assert_eq!(val, -500);
        } else {
            panic!("Expected Int16 value");
        }
    }

    #[test]
    fn test_parse_uint16_attribute() {
        let input = "UInt16 port_number 8080;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::UInt16);
        assert_eq!(attr.name, "port_number");
        if let DataValue::UInt16(val) = attr.value {
            assert_eq!(val, 8080);
        } else {
            panic!("Expected UInt16 value");
        }
    }

    #[test]
    fn test_parse_uint32_attribute() {
        let input = "UInt32 file_size 4294967295;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::UInt32);
        assert_eq!(attr.name, "file_size");
        if let DataValue::UInt32(val) = attr.value {
            assert_eq!(val, 4294967295);
        } else {
            panic!("Expected UInt32 value");
        }
    }

    #[test]
    fn test_parse_float64_attribute() {
        let input = "Float64 precision_value 3.141592653589793;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::Float64);
        assert_eq!(attr.name, "precision_value");
        if let DataValue::Float64(val) = attr.value {
            assert!((val - 3.141592653589793).abs() < 1e-15);
        } else {
            panic!("Expected Float64 value");
        }
    }

    #[test]
    fn test_parse_url_attribute() {
        let input = r#"URL data_source "http://example.com/data.nc";"#;
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::URL);
        assert_eq!(attr.name, "data_source");
        if let DataValue::URL(val) = attr.value {
            assert_eq!(val, "http://example.com/data.nc");
        } else {
            panic!("Expected URL value");
        }
    }

    #[test]
    fn test_das_variable_with_new_types() {
        let input = r#"    sensor_data {
        Byte quality_flag 1;
        Int16 elevation -500;
        UInt16 port_number 8080;
        UInt32 file_size 4294967295;
        Float64 precision_value 3.141592653589793;
        URL data_source "http://example.com/data.nc";
    }"#;

        let (_, (name, attrs)) = parse_das_variable(input).unwrap();
        assert_eq!(name, "sensor_data");
        assert_eq!(attrs.len(), 6);

        // Check each attribute type
        assert_eq!(attrs["quality_flag"].data_type, DataType::Byte);
        assert_eq!(attrs["elevation"].data_type, DataType::Int16);
        assert_eq!(attrs["port_number"].data_type, DataType::UInt16);
        assert_eq!(attrs["file_size"].data_type, DataType::UInt32);
        assert_eq!(attrs["precision_value"].data_type, DataType::Float64);
        assert_eq!(attrs["data_source"].data_type, DataType::URL);
    }
}

#[cfg(test)]
mod dds_array_tests {
    use super::*;

    #[test]
    fn test_parse_byte_array() {
        let input = "Byte quality_flags[time = 10];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::Byte);
        assert_eq!(array.name, "quality_flags");
        assert_eq!(array.coords.len(), 1);
        assert_eq!(array.coords[0].0, "time");
        assert_eq!(array.coords[0].1, 10);
        assert_eq!(array.array_length(), 10);
        assert_eq!(array.byte_count(), 8 + 10 * 1); // 8 bytes header + 10 bytes data
    }

    #[test]
    fn test_parse_int16_array() {
        let input = "Int16 elevations[latitude = 5][longitude = 5];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::Int16);
        assert_eq!(array.name, "elevations");
        assert_eq!(array.coords.len(), 2);
        assert_eq!(array.array_length(), 25);
        assert_eq!(array.byte_count(), 8 + 25 * 2); // 8 bytes header + 50 bytes data
    }

    #[test]
    fn test_parse_uint16_array() {
        let input = "UInt16 port_numbers[servers = 3];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::UInt16);
        assert_eq!(array.name, "port_numbers");
        assert_eq!(array.coords.len(), 1);
        assert_eq!(array.array_length(), 3);
        assert_eq!(array.byte_count(), 8 + 3 * 2); // 8 bytes header + 6 bytes data
    }

    #[test]
    fn test_parse_uint32_array() {
        let input = "UInt32 file_sizes[files = 100];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::UInt32);
        assert_eq!(array.name, "file_sizes");
        assert_eq!(array.coords.len(), 1);
        assert_eq!(array.array_length(), 100);
        assert_eq!(array.byte_count(), 8 + 100 * 4); // 8 bytes header + 400 bytes data
    }

    #[test]
    fn test_parse_float64_array() {
        let input = "Float64 precise_measurements[time = 1000];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::Float64);
        assert_eq!(array.name, "precise_measurements");
        assert_eq!(array.coords.len(), 1);
        assert_eq!(array.array_length(), 1000);
        assert_eq!(array.byte_count(), 8 + 1000 * 8); // 8 bytes header + 8000 bytes data
    }

    #[test]
    fn test_multidimensional_arrays() {
        let test_cases = vec![
            ("Byte data[x = 10][y = 20][z = 5];", DataType::Byte, 1000, 8 + 1000),
            ("Int16 data[x = 5][y = 5];", DataType::Int16, 25, 8 + 50),
            ("UInt16 data[x = 3][y = 3][z = 3];", DataType::UInt16, 27, 8 + 54),
            ("UInt32 data[x = 2][y = 2];", DataType::UInt32, 4, 8 + 16),
            ("Float64 data[x = 10];", DataType::Float64, 10, 8 + 80),
        ];

        for (input, expected_type, expected_length, expected_bytes) in test_cases {
            let (_, array) = DdsArray::parse(input).unwrap();
            assert_eq!(array.data_type, expected_type);
            assert_eq!(array.array_length(), expected_length);
            assert_eq!(array.byte_count(), expected_bytes);
        }
    }
}

#[cfg(test)]
mod data_array_conversion_tests {
    use super::*;

    #[test]
    fn test_data_array_to_i32_conversions() {
        // Test Byte array conversion
        let byte_array = DataArray::Byte(vec![-128, 0, 127]);
        let converted: Vec<i32> = byte_array.try_into().unwrap();
        assert_eq!(converted, vec![-128, 0, 127]);

        // Test Int16 array conversion
        let int16_array = DataArray::Int16(vec![-32768, 0, 32767]);
        let converted: Vec<i32> = int16_array.try_into().unwrap();
        assert_eq!(converted, vec![-32768, 0, 32767]);

        // Test UInt16 array conversion
        let uint16_array = DataArray::UInt16(vec![0, 32768, 65535]);
        let converted: Vec<i32> = uint16_array.try_into().unwrap();
        assert_eq!(converted, vec![0, 32768, 65535]);

        // Test UInt32 array conversion (with overflow)
        let uint32_array = DataArray::UInt32(vec![0, 2147483647, 4294967295]);
        let converted: Vec<i32> = uint32_array.try_into().unwrap();
        assert_eq!(converted, vec![0, 2147483647, -1]); // Last value overflows
    }

    #[test]
    fn test_data_array_to_f64_conversions() {
        // Test all numeric types converting to f64
        let byte_array = DataArray::Byte(vec![-1, 0, 1]);
        let converted: Vec<f64> = byte_array.try_into().unwrap();
        assert_eq!(converted, vec![-1.0, 0.0, 1.0]);

        let float64_array = DataArray::Float64(vec![3.14159265359, -2.71828, 0.0]);
        let converted: Vec<f64> = float64_array.try_into().unwrap();
        assert_eq!(converted, vec![3.14159265359, -2.71828, 0.0]);
    }

    #[test]
    fn test_invalid_data_array_conversions() {
        let string_array = DataArray::String(vec!["test".to_string()]);
        assert!(TryInto::<Vec<i32>>::try_into(string_array.clone()).is_err());
        assert!(TryInto::<Vec<f32>>::try_into(string_array.clone()).is_err());
        assert!(TryInto::<Vec<f64>>::try_into(string_array).is_err());

        let url_array = DataArray::URL(vec!["http://example.com".to_string()]);
        assert!(TryInto::<Vec<i32>>::try_into(url_array.clone()).is_err());
        assert!(TryInto::<Vec<f32>>::try_into(url_array.clone()).is_err());
        assert!(TryInto::<Vec<f64>>::try_into(url_array).is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_das_with_new_types() {
        let input = r#"Attributes {
    sensor_metadata {
        Byte quality_level 2;
        Int16 min_elevation -1000;
        UInt16 max_elevation 8848;
        UInt32 total_measurements 1000000;
        Float64 calibration_factor 1.23456789012345;
        URL documentation "https://example.com/docs";
        String instrument_name "Advanced Sensor";
    }
    measurement_data {
        Float32 _FillValue 999.0;
        Int32 valid_range_min -1000;
        Int32 valid_range_max 1000;
    }
}"#;

        let attrs = parse_das_attributes(input).unwrap();
        assert_eq!(attrs.len(), 2);
        
        // Check sensor_metadata
        let sensor_meta = &attrs["sensor_metadata"];
        assert_eq!(sensor_meta.len(), 7);
        assert_eq!(sensor_meta["quality_level"].data_type, DataType::Byte);
        assert_eq!(sensor_meta["min_elevation"].data_type, DataType::Int16);
        assert_eq!(sensor_meta["max_elevation"].data_type, DataType::UInt16);
        assert_eq!(sensor_meta["total_measurements"].data_type, DataType::UInt32);
        assert_eq!(sensor_meta["calibration_factor"].data_type, DataType::Float64);
        assert_eq!(sensor_meta["documentation"].data_type, DataType::URL);
        assert_eq!(sensor_meta["instrument_name"].data_type, DataType::String);

        // Check measurement_data
        let measurement_data = &attrs["measurement_data"];
        assert_eq!(measurement_data.len(), 3);
        assert_eq!(measurement_data["_FillValue"].data_type, DataType::Float32);
        assert_eq!(measurement_data["valid_range_min"].data_type, DataType::Int32);
        assert_eq!(measurement_data["valid_range_max"].data_type, DataType::Int32);
    }

    #[test]
    fn test_complete_dds_with_new_types() {
        let input = r#"Dataset {
    Byte quality_flags[time = 100];
    Int16 elevations[latitude = 180][longitude = 360];
    UInt16 port_numbers[servers = 10];
    UInt32 file_sizes[files = 1000];
    Float64 precise_coords[points = 50];
    Int32 timestamps[time = 100];
    Float32 temperatures[time = 100];
} test_dataset;"#;

        let dataset = DdsDataset::from_bytes(input).unwrap();
        assert_eq!(dataset.name, "test_dataset");
        assert_eq!(dataset.values.len(), 7);

        // Check each array type
        let quality_flags = dataset.values[0].array().unwrap();
        assert_eq!(quality_flags.data_type, DataType::Byte);
        assert_eq!(quality_flags.array_length(), 100);

        let elevations = dataset.values[1].array().unwrap();
        assert_eq!(elevations.data_type, DataType::Int16);
        assert_eq!(elevations.array_length(), 180 * 360);

        let port_numbers = dataset.values[2].array().unwrap();
        assert_eq!(port_numbers.data_type, DataType::UInt16);
        assert_eq!(port_numbers.array_length(), 10);

        let file_sizes = dataset.values[3].array().unwrap();
        assert_eq!(file_sizes.data_type, DataType::UInt32);
        assert_eq!(file_sizes.array_length(), 1000);

        let precise_coords = dataset.values[4].array().unwrap();
        assert_eq!(precise_coords.data_type, DataType::Float64);
        assert_eq!(precise_coords.array_length(), 50);
    }
}

#[cfg(test)]
mod structure_tests {
    use super::*;

    #[test]
    fn test_parse_simple_structure() {
        let input = r#"Structure {
    Int32 id;
    Float32 value;
    String name;
} sensor_data;"#;

        let (_, structure) = DdsStructure::parse(input).unwrap();
        assert_eq!(structure.name, "sensor_data");
        assert_eq!(structure.fields.len(), 3);

        // Check field types
        assert!(matches!(structure.fields[0], DdsValue::Array(_)));
        assert!(matches!(structure.fields[1], DdsValue::Array(_)));
        assert!(matches!(structure.fields[2], DdsValue::Array(_)));

        let id_array = structure.fields[0].array().unwrap();
        assert_eq!(id_array.data_type, DataType::Int32);
        assert_eq!(id_array.name, "id");

        let value_array = structure.fields[1].array().unwrap();
        assert_eq!(value_array.data_type, DataType::Float32);
        assert_eq!(value_array.name, "value");

        let name_array = structure.fields[2].array().unwrap();
        assert_eq!(name_array.data_type, DataType::String);
        assert_eq!(name_array.name, "name");
    }

    #[test]
    fn test_parse_nested_structure() {
        let input = r#"Structure {
    Int32 outer_id;
    Structure {
        Float32 inner_value;
        String inner_name;
    } inner_struct;
} nested_data;"#;

        let (_, structure) = DdsStructure::parse(input).unwrap();
        assert_eq!(structure.name, "nested_data");
        assert_eq!(structure.fields.len(), 2);

        // Check outer field
        let outer_id = structure.fields[0].array().unwrap();
        assert_eq!(outer_id.data_type, DataType::Int32);
        assert_eq!(outer_id.name, "outer_id");

        // Check nested structure
        let inner_struct = structure.fields[1].structure().unwrap();
        assert_eq!(inner_struct.name, "inner_struct");
        assert_eq!(inner_struct.fields.len(), 2);

        let inner_value = inner_struct.fields[0].array().unwrap();
        assert_eq!(inner_value.data_type, DataType::Float32);
        assert_eq!(inner_value.name, "inner_value");

        let inner_name = inner_struct.fields[1].array().unwrap();
        assert_eq!(inner_name.data_type, DataType::String);
        assert_eq!(inner_name.name, "inner_name");
    }

    #[test]
    fn test_structure_with_arrays() {
        let input = r#"Structure {
    Int32 timestamps[time = 100];
    Float32 measurements[time = 100][sensors = 5];
    Byte quality_flags[time = 100];
} time_series;"#;

        let (_, structure) = DdsStructure::parse(input).unwrap();
        assert_eq!(structure.name, "time_series");
        assert_eq!(structure.fields.len(), 3);

        let timestamps = structure.fields[0].array().unwrap();
        assert_eq!(timestamps.data_type, DataType::Int32);
        assert_eq!(timestamps.array_length(), 100);

        let measurements = structure.fields[1].array().unwrap();
        assert_eq!(measurements.data_type, DataType::Float32);
        assert_eq!(measurements.array_length(), 500); // 100 * 5

        let quality_flags = structure.fields[2].array().unwrap();
        assert_eq!(quality_flags.data_type, DataType::Byte);
        assert_eq!(quality_flags.array_length(), 100);
    }
}

#[cfg(test)]
mod sequence_tests {
    use super::*;

    #[test]
    fn test_parse_simple_sequence() {
        let input = r#"Sequence {
    Int32 record_id;
    Float32 temperature;
    String location;
} weather_records;"#;

        let (_, sequence) = DdsSequence::parse(input).unwrap();
        assert_eq!(sequence.name, "weather_records");
        assert_eq!(sequence.fields.len(), 3);

        // Check field types
        let record_id = sequence.fields[0].array().unwrap();
        assert_eq!(record_id.data_type, DataType::Int32);
        assert_eq!(record_id.name, "record_id");

        let temperature = sequence.fields[1].array().unwrap();
        assert_eq!(temperature.data_type, DataType::Float32);
        assert_eq!(temperature.name, "temperature");

        let location = sequence.fields[2].array().unwrap();
        assert_eq!(location.data_type, DataType::String);
        assert_eq!(location.name, "location");
    }

    #[test]
    fn test_parse_sequence_with_structure() {
        let input = r#"Sequence {
    Int32 id;
    Structure {
        Float32 x;
        Float32 y;
        Float32 z;
    } coordinates;
    String description;
} spatial_data;"#;

        let (_, sequence) = DdsSequence::parse(input).unwrap();
        assert_eq!(sequence.name, "spatial_data");
        assert_eq!(sequence.fields.len(), 3);

        // Check ID field
        let id = sequence.fields[0].array().unwrap();
        assert_eq!(id.data_type, DataType::Int32);
        assert_eq!(id.name, "id");

        // Check nested structure
        let coordinates = sequence.fields[1].structure().unwrap();
        assert_eq!(coordinates.name, "coordinates");
        assert_eq!(coordinates.fields.len(), 3);

        let x = coordinates.fields[0].array().unwrap();
        assert_eq!(x.data_type, DataType::Float32);
        assert_eq!(x.name, "x");

        // Check description field
        let description = sequence.fields[2].array().unwrap();
        assert_eq!(description.data_type, DataType::String);
        assert_eq!(description.name, "description");
    }

    #[test]
    fn test_parse_nested_sequences() {
        let input = r#"Sequence {
    Int32 group_id;
    Sequence {
        Int32 item_id;
        Float32 value;
    } items;
} grouped_data;"#;

        let (_, sequence) = DdsSequence::parse(input).unwrap();
        assert_eq!(sequence.name, "grouped_data");
        assert_eq!(sequence.fields.len(), 2);

        // Check group_id field
        let group_id = sequence.fields[0].array().unwrap();
        assert_eq!(group_id.data_type, DataType::Int32);
        assert_eq!(group_id.name, "group_id");

        // Check nested sequence
        let items = sequence.fields[1].sequence().unwrap();
        assert_eq!(items.name, "items");
        assert_eq!(items.fields.len(), 2);

        let item_id = items.fields[0].array().unwrap();
        assert_eq!(item_id.data_type, DataType::Int32);
        assert_eq!(item_id.name, "item_id");

        let value = items.fields[1].array().unwrap();
        assert_eq!(value.data_type, DataType::Float32);
        assert_eq!(value.name, "value");
    }
}

#[cfg(test)]
mod complex_integration_tests {
    use super::*;

    #[test]
    fn test_dataset_with_all_constructor_types() {
        let input = r#"Dataset {
    Int32 simple_array[time = 10];
    Grid {
     ARRAY:
        Float32 temperature[time = 10][lat = 5][lon = 5];
     MAPS:
        Int32 time[time = 10];
        Float32 lat[lat = 5];
        Float32 lon[lon = 5];
    } temperature_grid;
    Structure {
        Int32 station_id;
        Float32 elevation;
        String name;
    } station_info;
    Sequence {
        Int32 measurement_id;
        Float64 timestamp;
        Float32 value;
        Byte quality_flag;
    } measurements;
} comprehensive_dataset;"#;

        let dataset = DdsDataset::from_bytes(input).unwrap();
        assert_eq!(dataset.name, "comprehensive_dataset");
        assert_eq!(dataset.values.len(), 4);

        // Check simple array
        let simple_array = dataset.values[0].array().unwrap();
        assert_eq!(simple_array.data_type, DataType::Int32);
        assert_eq!(simple_array.name, "simple_array");

        // Check grid
        let grid = dataset.values[1].grid().unwrap();
        assert_eq!(grid.name, "temperature_grid");
        assert_eq!(grid.array.data_type, DataType::Float32);

        // Check structure
        let structure = dataset.values[2].structure().unwrap();
        assert_eq!(structure.name, "station_info");
        assert_eq!(structure.fields.len(), 3);

        // Check sequence
        let sequence = dataset.values[3].sequence().unwrap();
        assert_eq!(sequence.name, "measurements");
        assert_eq!(sequence.fields.len(), 4);

        // Verify sequence field types
        let measurement_id = sequence.fields[0].array().unwrap();
        assert_eq!(measurement_id.data_type, DataType::Int32);

        let timestamp = sequence.fields[1].array().unwrap();
        assert_eq!(timestamp.data_type, DataType::Float64);

        let value = sequence.fields[2].array().unwrap();
        assert_eq!(value.data_type, DataType::Float32);

        let quality_flag = sequence.fields[3].array().unwrap();
        assert_eq!(quality_flag.data_type, DataType::Byte);
    }

    #[test]
    fn test_error_handling_for_type_casting() {
        let input = r#"Dataset {
    Structure {
        Int32 id;
        String name;
    } metadata;
} test_dataset;"#;

        let dataset = DdsDataset::from_bytes(input).unwrap();
        let structure_value = &dataset.values[0];

        // These should fail because it's a structure, not an array or grid
        assert!(structure_value.array().is_err());
        assert!(structure_value.grid().is_err());
        assert!(structure_value.sequence().is_err());

        // This should succeed
        assert!(structure_value.structure().is_ok());
    }
}