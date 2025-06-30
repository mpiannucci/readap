use readap::url::{
    ConstraintBuilder, CoordinateResolver, IndexSelection, OpenDAPUrlBuilder, ValueSelection,
};
use std::collections::HashMap;

#[test]
fn test_opendap_url_builder_basic() {
    let builder = OpenDAPUrlBuilder::new("http://example.com/data");

    assert_eq!(builder.base_url(), "http://example.com/data");
    assert_eq!(builder.das_url(), "http://example.com/data.das");
    assert_eq!(builder.dds_url(), "http://example.com/data.dds");
    assert_eq!(builder.dods_url(None), "http://example.com/data.dods");
}

#[test]
fn test_opendap_url_builder_with_nc_extension() {
    let builder = OpenDAPUrlBuilder::new("http://example.com/data.nc");

    assert_eq!(builder.base_url(), "http://example.com/data");
    assert_eq!(builder.das_url(), "http://example.com/data.das");
    assert_eq!(builder.dds_url(), "http://example.com/data.dds");
    assert_eq!(builder.dods_url(None), "http://example.com/data.dods");
}

#[test]
fn test_dods_url_with_constraints() {
    let builder = OpenDAPUrlBuilder::new("http://example.com/data");

    assert_eq!(
        builder.dods_url(Some("temp[0:10],pressure[5]")),
        "http://example.com/data.dods?temp[0:10],pressure[5]"
    );

    assert_eq!(builder.dods_url(Some("")), "http://example.com/data.dods");
}

#[test]
fn test_constraint_builder_empty() {
    let builder = ConstraintBuilder::new();
    assert_eq!(builder.build(), "");
}

#[test]
fn test_constraint_builder_single_index() {
    let mut selections = HashMap::new();
    selections.insert("temperature".to_string(), IndexSelection::Single(5));

    let builder = ConstraintBuilder::new().isel(selections);
    assert_eq!(builder.build(), "temperature[5]");
}

#[test]
fn test_constraint_builder_range_index() {
    let mut selections = HashMap::new();
    selections.insert("temperature".to_string(), IndexSelection::Range(0, 10));

    let builder = ConstraintBuilder::new().isel(selections);
    assert_eq!(builder.build(), "temperature[0:10]");
}

#[test]
fn test_constraint_builder_stride_index() {
    let mut selections = HashMap::new();
    selections.insert("temperature".to_string(), IndexSelection::Stride(0, 2, 10));

    let builder = ConstraintBuilder::new().isel(selections);
    assert_eq!(builder.build(), "temperature[0:2:10]");
}

#[test]
fn test_constraint_builder_multiple_indices() {
    let mut selections = HashMap::new();
    selections.insert(
        "temperature".to_string(),
        IndexSelection::Multiple(vec![0, 5, 10]),
    );

    let builder = ConstraintBuilder::new().isel(selections);
    assert_eq!(builder.build(), "temperature[0][5][10]");
}

#[test]
fn test_constraint_builder_multiple_variables() {
    let mut temp_selections = HashMap::new();
    temp_selections.insert("temperature".to_string(), IndexSelection::Range(0, 10));

    let mut pressure_selections = HashMap::new();
    pressure_selections.insert("pressure".to_string(), IndexSelection::Single(5));

    let builder = ConstraintBuilder::new()
        .isel(temp_selections)
        .isel(pressure_selections);

    let constraint_str = builder.build();

    // Should contain both variables (order may vary)
    assert!(constraint_str.contains("temperature[0:10]"));
    assert!(constraint_str.contains("pressure[5]"));
    assert!(constraint_str.contains(","));
}

#[test]
fn test_constraint_builder_chaining_same_variable() {
    let mut first_selections = HashMap::new();
    first_selections.insert("temperature".to_string(), IndexSelection::Range(0, 10));

    let mut second_selections = HashMap::new();
    second_selections.insert("temperature".to_string(), IndexSelection::Single(5));

    let builder = ConstraintBuilder::new()
        .isel(first_selections)
        .isel(second_selections);

    let constraint_str = builder.build();

    // Should have both constraints on the same variable
    assert!(constraint_str.contains("temperature[0:10][5]"));
}

#[test]
fn test_url_builder_with_constraint_builder() {
    let url_builder = OpenDAPUrlBuilder::new("http://example.com/data");

    let mut selections = HashMap::new();
    selections.insert("temperature".to_string(), IndexSelection::Range(0, 10));
    selections.insert("pressure".to_string(), IndexSelection::Single(5));

    let constraint_builder = ConstraintBuilder::new().isel(selections);
    let dods_url = url_builder.dods_url_with_constraints(&constraint_builder);

    assert!(dods_url.starts_with("http://example.com/data.dods?"));
    assert!(dods_url.contains("temperature[0:10]"));
    assert!(dods_url.contains("pressure[5]"));
}

#[test]
fn test_coordinate_resolver_empty() {
    let resolver = CoordinateResolver::new();
    let builder = ConstraintBuilder::new();

    let resolved = resolver.resolve_constraints(&builder).unwrap();
    assert_eq!(resolved.build(), "");
}

#[test]
fn test_coordinate_resolver_missing_coordinates() {
    let resolver = CoordinateResolver::new();

    let mut selections = HashMap::new();
    selections.insert("temperature".to_string(), ValueSelection::Single(25.0));

    let builder = ConstraintBuilder::new().sel(selections);

    let result = resolver.resolve_constraints(&builder);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No coordinates found"));
}

#[test]
fn test_coordinate_resolver_single_value() {
    let mut resolver = CoordinateResolver::new();
    resolver.add_coordinates("temperature".to_string(), vec![20.0, 25.0, 30.0, 35.0]);

    let mut selections = HashMap::new();
    selections.insert("temperature".to_string(), ValueSelection::Single(27.0));

    let builder = ConstraintBuilder::new().sel(selections);
    let resolved = resolver.resolve_constraints(&builder).unwrap();

    // 27.0 should resolve to index 1 (nearest to 25.0) or 2 (nearest to 30.0)
    // Let's check it resolves to a valid index constraint
    let constraint_str = resolved.build();
    assert!(constraint_str.contains("temperature["));
    assert!(constraint_str.contains("]"));
}

#[test]
fn test_coordinate_resolver_value_range() {
    let mut resolver = CoordinateResolver::new();
    resolver.add_coordinates(
        "temperature".to_string(),
        vec![20.0, 25.0, 30.0, 35.0, 40.0],
    );

    let mut selections = HashMap::new();
    selections.insert("temperature".to_string(), ValueSelection::Range(23.0, 37.0));

    let builder = ConstraintBuilder::new().sel(selections);
    let resolved = resolver.resolve_constraints(&builder).unwrap();

    let constraint_str = resolved.build();
    assert!(constraint_str.contains("temperature["));
    assert!(constraint_str.contains(":"));
    assert!(constraint_str.contains("]"));
}

#[test]
fn test_coordinate_resolver_multiple_values() {
    let mut resolver = CoordinateResolver::new();
    resolver.add_coordinates(
        "temperature".to_string(),
        vec![20.0, 25.0, 30.0, 35.0, 40.0],
    );

    let mut selections = HashMap::new();
    selections.insert(
        "temperature".to_string(),
        ValueSelection::Multiple(vec![22.0, 33.0]),
    );

    let builder = ConstraintBuilder::new().sel(selections);
    let resolved = resolver.resolve_constraints(&builder).unwrap();

    let constraint_str = resolved.build();
    assert!(constraint_str.contains("temperature["));
    // Should have multiple index selections
    let count = constraint_str.matches("[").count();
    assert!(count >= 2);
}

#[test]
fn test_coordinate_resolver_mixed_constraints() {
    let mut resolver = CoordinateResolver::new();
    resolver.add_coordinates("temperature".to_string(), vec![20.0, 25.0, 30.0, 35.0]);
    resolver.add_coordinates("pressure".to_string(), vec![1000.0, 900.0, 800.0, 700.0]);

    // Mix of index and value selections
    let mut index_selections = HashMap::new();
    index_selections.insert("height".to_string(), IndexSelection::Range(0, 10));

    let mut value_selections = HashMap::new();
    value_selections.insert("temperature".to_string(), ValueSelection::Single(27.0));
    value_selections.insert("pressure".to_string(), ValueSelection::Range(750.0, 950.0));

    let builder = ConstraintBuilder::new()
        .isel(index_selections)
        .sel(value_selections);

    let resolved = resolver.resolve_constraints(&builder).unwrap();
    let constraint_str = resolved.build();

    // Should contain all three variables with appropriate constraints
    assert!(constraint_str.contains("height[0:10]"));
    assert!(constraint_str.contains("temperature["));
    assert!(constraint_str.contains("pressure["));
}

#[test]
fn test_nearest_neighbor_edge_cases() {
    use readap::url::find_nearest_index;

    let coords = vec![0.0, 1.0, 2.0, 3.0, 4.0];

    // Test exact matches
    assert_eq!(find_nearest_index(&coords, 0.0).unwrap(), 0);
    assert_eq!(find_nearest_index(&coords, 2.0).unwrap(), 2);
    assert_eq!(find_nearest_index(&coords, 4.0).unwrap(), 4);

    // Test values outside range
    assert_eq!(find_nearest_index(&coords, -1.0).unwrap(), 0);
    assert_eq!(find_nearest_index(&coords, 10.0).unwrap(), 4);

    // Test midpoint values
    assert_eq!(find_nearest_index(&coords, 0.5).unwrap(), 1);
    assert_eq!(find_nearest_index(&coords, 1.5).unwrap(), 2);

    // Test empty coordinates
    let empty_coords: Vec<f64> = vec![];
    assert!(find_nearest_index(&empty_coords, 1.0).is_err());
}

#[test]
fn test_real_world_oceanographic_example() {
    let mut resolver = CoordinateResolver::new();

    // Typical ocean model coordinates
    resolver.add_coordinates("time".to_string(), vec![0.0, 6.0, 12.0, 18.0, 24.0]); // hours
    resolver.add_coordinates("depth".to_string(), vec![0.0, 10.0, 20.0, 50.0, 100.0]); // meters
    resolver.add_coordinates("lat".to_string(), vec![40.0, 40.5, 41.0, 41.5, 42.0]); // degrees
    resolver.add_coordinates("lon".to_string(), vec![-74.0, -73.5, -73.0, -72.5, -72.0]); // degrees

    // Select data for a specific time and location
    let mut selections = HashMap::new();
    selections.insert("time".to_string(), ValueSelection::Single(15.0)); // closest to 12.0 or 18.0
    selections.insert("depth".to_string(), ValueSelection::Range(5.0, 75.0)); // from surface to ~50m
    selections.insert("lat".to_string(), ValueSelection::Single(40.7)); // closest to 40.5 or 41.0
    selections.insert("lon".to_string(), ValueSelection::Single(-73.2)); // closest to -73.0 or -73.5

    let builder = ConstraintBuilder::new().sel(selections);
    let resolved = resolver.resolve_constraints(&builder).unwrap();
    let constraint_str = resolved.build();

    // Verify all coordinates are resolved to indices
    assert!(constraint_str.contains("time["));
    assert!(constraint_str.contains("depth["));
    assert!(constraint_str.contains("lat["));
    assert!(constraint_str.contains("lon["));

    // Should not contain any VALUE_LOOKUP_NEEDED placeholders
    assert!(!constraint_str.contains("VALUE_LOOKUP_NEEDED"));

    println!("Resolved oceanographic constraint: {}", constraint_str);
}

#[test]
fn test_gridded_coordinate_handling() {
    // Test case for gridded coordinates (like curvilinear grids)
    let mut resolver = CoordinateResolver::new();

    // In real gridded data, lat/lon might vary by both i,j indices
    // For this test, we'll simulate 1D coordinate arrays
    resolver.add_coordinates("x".to_string(), vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    resolver.add_coordinates("y".to_string(), vec![0.0, 0.5, 1.0, 1.5, 2.0]);

    let mut selections = HashMap::new();
    selections.insert("x".to_string(), ValueSelection::Range(0.5, 3.5));
    selections.insert("y".to_string(), ValueSelection::Single(1.2));

    let builder = ConstraintBuilder::new().sel(selections);
    let resolved = resolver.resolve_constraints(&builder).unwrap();
    let constraint_str = resolved.build();

    // Should resolve to appropriate index ranges
    assert!(constraint_str.contains("x["));
    assert!(constraint_str.contains("y["));

    println!("Resolved gridded constraint: {}", constraint_str);
}
