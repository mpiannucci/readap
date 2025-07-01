use readap::{CoordinateConstraint, DdsDataset, QueryError};

#[test]
fn test_real_dataset_query_basic() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test basic variable selection
    let url = dataset
        .query("https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml")
        .select_variable("wind_dir")
        .unwrap()
        .dods_url()
        .unwrap();

    assert_eq!(
        url,
        "https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml.dods?wind_dir"
    );
}

#[test]
fn test_real_dataset_multiple_variables() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    let url = dataset
        .query("https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml")
        .select_variables(&["wind_dir", "wind_spd", "air_temperature"])
        .unwrap()
        .dods_url()
        .unwrap();

    assert_eq!(url, "https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml.dods?wind_dir,wind_spd,air_temperature");
}

#[test]
fn test_real_dataset_coordinate_constraints() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test time series subset
    let url = dataset
        .query("https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml")
        .select_variable("sea_surface_temperature")
        .unwrap()
        .select_by_coordinate("time", CoordinateConstraint::range(0, 1000))
        .unwrap()
        .select_by_coordinate("latitude", CoordinateConstraint::single(0))
        .unwrap()
        .select_by_coordinate("longitude", CoordinateConstraint::single(0))
        .unwrap()
        .dods_url()
        .unwrap();

    assert_eq!(url, "https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml.dods?sea_surface_temperature[0:1000][0][0]");
}

#[test]
fn test_real_dataset_strided_access() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test every 10th time step
    let url = dataset
        .query("https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml")
        .select_variable("wave_height")
        .unwrap()
        .select_by_coordinate("time", CoordinateConstraint::range_with_stride(0, 100, 10))
        .unwrap()
        .select_by_coordinate("latitude", CoordinateConstraint::single(0))
        .unwrap()
        .select_by_coordinate("longitude", CoordinateConstraint::single(0))
        .unwrap()
        .dods_url()
        .unwrap();

    assert_eq!(url, "https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml.dods?wave_height[0:10:100][0][0]");
}

#[test]
fn test_real_dataset_metadata_exploration() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test metadata methods
    let variables = dataset.list_variables();
    assert!(variables.len() > 10); // Should have many variables
    assert!(variables.contains(&"wind_dir".to_string()));
    assert!(variables.contains(&"wind_spd".to_string()));
    assert!(variables.contains(&"wave_height".to_string()));
    assert!(variables.contains(&"air_temperature".to_string()));
    assert!(variables.contains(&"sea_surface_temperature".to_string()));

    let coordinates = dataset.list_coordinates();
    assert_eq!(coordinates.len(), 3);
    assert!(coordinates.contains(&"time".to_string()));
    assert!(coordinates.contains(&"latitude".to_string()));
    assert!(coordinates.contains(&"longitude".to_string()));

    // Test variable info
    let wind_info = dataset.get_variable_info("wind_dir").unwrap();
    assert_eq!(wind_info.name, "wind_dir");
    assert_eq!(wind_info.coordinates, vec!["time", "latitude", "longitude"]);
    assert_eq!(wind_info.dimensions.len(), 3);
    assert_eq!(wind_info.dimensions[0].0, "time");
    assert_eq!(wind_info.dimensions[0].1, 461468); // Large time dimension

    // Test coordinate info
    let time_info = dataset.get_coordinate_info("time").unwrap();
    assert_eq!(time_info.name, "time");
    assert_eq!(time_info.size, 461468);
    assert!(time_info.variables_using.len() > 10); // Many variables use time
}

#[test]
fn test_real_dataset_validation_errors() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test variable not found
    let result = dataset
        .query("https://example.com/data")
        .select_variable("nonexistent_variable");
    assert!(matches!(result, Err(QueryError::VariableNotFound(_))));

    // Test coordinate not found
    let result = dataset
        .query("https://example.com/data")
        .select_variable("wind_dir")
        .unwrap()
        .select_by_coordinate("nonexistent_coord", CoordinateConstraint::single(0));
    assert!(matches!(result, Err(QueryError::CoordinateNotFound(_))));

    // Test index out of bounds
    let result = dataset
        .query("https://example.com/data")
        .select_variable("wind_dir")
        .unwrap()
        .select_by_coordinate("latitude", CoordinateConstraint::single(10)); // latitude only has 1 element
    assert!(matches!(result, Err(QueryError::IndexOutOfBounds(_, _, _))));

    // Test coordinate not available for variable (latitude array doesn't have time coordinate)
    let result = dataset
        .query("https://example.com/data")
        .select_variable("latitude")
        .unwrap()
        .select_by_coordinate("time", CoordinateConstraint::single(0));
    assert!(matches!(
        result,
        Err(QueryError::CoordinateNotAvailableForVariable(_, _))
    ));
}

#[test]
fn test_real_dataset_size_estimation() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test full variable size estimation
    let query = dataset
        .query("https://example.com/data")
        .select_variable("wind_dir")
        .unwrap();

    // wind_dir is Int32 (4 bytes) with dimensions [time=461468][latitude=1][longitude=1]
    let expected_size = 461468 * 1 * 1 * 4;
    assert_eq!(query.estimated_size(), expected_size);

    // Test subset size estimation
    let query = dataset
        .query("https://example.com/data")
        .select_variable("wind_dir")
        .unwrap()
        .select_by_coordinate("time", CoordinateConstraint::range(0, 99))
        .unwrap() // 100 time steps
        .select_by_coordinate("latitude", CoordinateConstraint::single(0))
        .unwrap()
        .select_by_coordinate("longitude", CoordinateConstraint::single(0))
        .unwrap();

    let expected_subset_size = 100 * 1 * 1 * 4;
    assert_eq!(query.estimated_size(), expected_subset_size);

    // Test strided size estimation
    let query = dataset
        .query("https://example.com/data")
        .select_variable("wind_dir")
        .unwrap()
        .select_by_coordinate("time", CoordinateConstraint::range_with_stride(0, 99, 10))
        .unwrap() // Every 10th step
        .select_by_coordinate("latitude", CoordinateConstraint::single(0))
        .unwrap()
        .select_by_coordinate("longitude", CoordinateConstraint::single(0))
        .unwrap();

    let expected_strided_size = 10 * 1 * 1 * 4; // 100 steps / 10 stride = 10 points
    assert_eq!(query.estimated_size(), expected_strided_size);
}

#[test]
fn test_real_dataset_complex_query() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test complex query with multiple variables and constraints
    let query = dataset
        .query("https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml")
        .select_variables(&["wind_dir", "wind_spd", "wave_height"])
        .unwrap()
        .select_by_coordinate(
            "time",
            CoordinateConstraint::range_with_stride(1000, 2000, 5),
        )
        .unwrap()
        .select_by_coordinate("latitude", CoordinateConstraint::single(0))
        .unwrap()
        .select_by_coordinate("longitude", CoordinateConstraint::single(0))
        .unwrap();

    // Validate the query
    assert!(query.validate().is_ok());

    // Check selected variables
    assert_eq!(
        query.selected_variables(),
        &["wind_dir", "wind_spd", "wave_height"]
    );

    // Check active constraints
    let constraints = query.active_constraints();
    assert_eq!(constraints.len(), 3);
    assert!(constraints.contains_key("time"));
    assert!(constraints.contains_key("latitude"));
    assert!(constraints.contains_key("longitude"));

    // Generate URL
    let url = query.dods_url().unwrap();
    let expected = "https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml.dods?wind_dir[1000:5:2000][0][0],wind_spd[1000:5:2000][0][0],wave_height[1000:5:2000][0][0]";
    assert_eq!(url, expected);
}

#[test]
fn test_convenience_methods() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test convenience constraint methods
    let url = dataset
        .query("https://example.com/data")
        .select_variable("air_temperature")
        .unwrap()
        .select_by_coordinate("time", CoordinateConstraint::first())
        .unwrap()
        .select_by_coordinate("latitude", CoordinateConstraint::single(0))
        .unwrap()
        .select_by_coordinate("longitude", CoordinateConstraint::single(0))
        .unwrap()
        .dods_url()
        .unwrap();

    assert_eq!(
        url,
        "https://example.com/data.dods?air_temperature[0][0][0]"
    );

    // Test last() method
    let url = dataset
        .query("https://example.com/data")
        .select_variable("air_temperature")
        .unwrap()
        .select_by_coordinate("time", CoordinateConstraint::last(461468))
        .unwrap() // Use actual size
        .select_by_coordinate("latitude", CoordinateConstraint::single(0))
        .unwrap()
        .select_by_coordinate("longitude", CoordinateConstraint::single(0))
        .unwrap()
        .dods_url()
        .unwrap();

    assert_eq!(
        url,
        "https://example.com/data.dods?air_temperature[461467][0][0]"
    ); // 461468 - 1 = 461467
}

#[test]
fn test_das_dds_url_generation() {
    let dds_content = include_str!("../data/44008.ncml.dds");
    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    let query = dataset
        .query("https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml")
        .select_variable("wind_dir")
        .unwrap();

    assert_eq!(
        query.das_url(),
        "https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml.das"
    );
    assert_eq!(
        query.dds_url(),
        "https://data.nodc.noaa.gov/thredds/dodsC/data/stdmet/44008/44008.ncml.dds"
    );
}

#[test]
fn test_swden_dataset() {
    // Test with the spectral wave density dataset
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

    let dataset = DdsDataset::from_bytes(dds_content).unwrap();

    // Test frequency-based subsetting
    let url = dataset
        .query("https://example.com/data")
        .select_variable("spectral_wave_density")
        .unwrap()
        .select_by_coordinate("time", CoordinateConstraint::range(0, 6))
        .unwrap() // All time steps
        .select_by_coordinate("frequency", CoordinateConstraint::range(10, 30))
        .unwrap() // Frequency subset
        .select_by_coordinate("latitude", CoordinateConstraint::single(0))
        .unwrap()
        .select_by_coordinate("longitude", CoordinateConstraint::single(0))
        .unwrap()
        .dods_url()
        .unwrap();

    assert_eq!(
        url,
        "https://example.com/data.dods?spectral_wave_density[0:6][10:30][0][0]"
    );

    // Test metadata
    let variables = dataset.list_variables();
    assert_eq!(variables.len(), 3);
    assert!(variables.contains(&"time".to_string()));
    assert!(variables.contains(&"frequency".to_string()));
    assert!(variables.contains(&"spectral_wave_density".to_string()));

    let coordinates = dataset.list_coordinates();
    assert_eq!(coordinates.len(), 4);
    assert!(coordinates.contains(&"time".to_string()));
    assert!(coordinates.contains(&"frequency".to_string()));
    assert!(coordinates.contains(&"latitude".to_string()));
    assert!(coordinates.contains(&"longitude".to_string()));
}
