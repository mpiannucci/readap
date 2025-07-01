//! High-level coordinate-aware query API for OpenDAP datasets
//!
//! This module provides a metadata-driven query builder that allows users to select variables
//! and apply coordinate constraints by name rather than anonymous indices. It leverages the
//! rich metadata available in DDS files to provide validation and type safety.
//!
//! # Examples
//!
//! ## Basic variable selection
//! ```
//! use readap::DdsDataset;
//!
//! # let dds_content = r#"Dataset {
//! #     Float32 latitude[latitude = 1];
//! #     Float32 longitude[longitude = 1];
//! #     Int32 time[time = 100];
//! #     Grid {
//! #      ARRAY:
//! #         Float32 temperature[time = 100][latitude = 1][longitude = 1];
//! #      MAPS:
//! #         Int32 time[time = 100];
//! #         Float32 latitude[latitude = 1];
//! #         Float32 longitude[longitude = 1];
//! #     } temperature;
//! # } test_dataset;"#;
//! # let dataset = DdsDataset::from_bytes(dds_content).unwrap();
//! let url = dataset.query("https://example.com/data")
//!     .select_variable("temperature").unwrap()
//!     .dods_url().unwrap();
//! ```
//!
//! ## Coordinate-based subsetting
//! ```
//! use readap::{DdsDataset, CoordinateConstraint};
//!
//! # let dds_content = r#"Dataset {
//! #     Float32 latitude[latitude = 1];
//! #     Float32 longitude[longitude = 1];
//! #     Int32 time[time = 100];
//! #     Grid {
//! #      ARRAY:
//! #         Float32 temperature[time = 100][latitude = 1][longitude = 1];
//! #      MAPS:
//! #         Int32 time[time = 100];
//! #         Float32 latitude[latitude = 1];
//! #         Float32 longitude[longitude = 1];
//! #     } temperature;
//! # } test_dataset;"#;
//! # let dataset = DdsDataset::from_bytes(dds_content).unwrap();
//! let url = dataset.query("https://example.com/data")
//!     .select_variable("temperature").unwrap()
//!     .select_by_coordinate("time", CoordinateConstraint::range(0, 50)).unwrap()
//!     .select_by_coordinate("latitude", CoordinateConstraint::single(0)).unwrap()
//!     .select_by_coordinate("longitude", CoordinateConstraint::single(0)).unwrap()
//!     .dods_url().unwrap();
//! ```

use crate::{data::DataType, dds::*, url_builder::*};
use std::collections::HashMap;
use thiserror::Error;

/// Coordinate constraint types for subsetting data
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinateConstraint {
    /// Index-based range constraint with optional stride
    Indices {
        start: usize,
        end: usize,
        stride: Option<usize>,
    },
    /// Single index constraint
    Single(usize),
}

impl CoordinateConstraint {
    /// Create a single index constraint
    pub fn single(index: usize) -> Self {
        CoordinateConstraint::Single(index)
    }

    /// Create a range constraint without stride
    pub fn range(start: usize, end: usize) -> Self {
        CoordinateConstraint::Indices {
            start,
            end,
            stride: None,
        }
    }

    /// Create a range constraint with stride
    pub fn range_with_stride(start: usize, end: usize, stride: usize) -> Self {
        CoordinateConstraint::Indices {
            start,
            end,
            stride: Some(stride),
        }
    }

    /// Create a constraint for the first index (0)
    pub fn first() -> Self {
        CoordinateConstraint::Single(0)
    }

    /// Create a constraint for the last index
    pub fn last(size: u32) -> Self {
        CoordinateConstraint::Single((size.saturating_sub(1)) as usize)
    }

    /// Validate the constraint against a coordinate size
    pub fn validate(&self, coord_name: &str, size: u32) -> Result<(), QueryError> {
        match self {
            CoordinateConstraint::Single(index) => {
                if *index >= size as usize {
                    return Err(QueryError::IndexOutOfBounds(
                        *index,
                        coord_name.to_string(),
                        size,
                    ));
                }
            }
            CoordinateConstraint::Indices { start, end, .. } => {
                if *start >= size as usize {
                    return Err(QueryError::IndexOutOfBounds(
                        *start,
                        coord_name.to_string(),
                        size,
                    ));
                }
                if *end >= size as usize {
                    return Err(QueryError::IndexOutOfBounds(
                        *end,
                        coord_name.to_string(),
                        size,
                    ));
                }
                if start > end {
                    return Err(QueryError::InvalidCoordinateRange(
                        format!("Start index {start} is greater than end index {end} for coordinate '{coord_name}'")
                    ));
                }
            }
        }
        Ok(())
    }

    /// Convert to IndexRange for UrlBuilder
    pub fn to_index_ranges(&self) -> Vec<IndexRange> {
        match self {
            CoordinateConstraint::Single(index) => vec![IndexRange::Single(*index as isize)],
            CoordinateConstraint::Indices { start, end, stride } => {
                vec![IndexRange::Range {
                    start: *start as isize,
                    end: *end as isize,
                    stride: stride.map(|s| s as isize),
                }]
            }
        }
    }
}

/// Variable type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Array,
    Grid,
    Structure,
    Sequence,
}

/// Metadata information about a variable
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub data_type: DataType,
    pub coordinates: Vec<String>,
    pub dimensions: Vec<(String, u32)>,
    pub variable_type: VariableType,
}

/// Metadata information about a coordinate
#[derive(Debug, Clone)]
pub struct CoordinateInfo {
    pub name: String,
    pub data_type: DataType,
    pub size: u32,
    pub variables_using: Vec<String>,
}

/// Query-specific error types
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Variable '{0}' not found in dataset")]
    VariableNotFound(String),
    #[error("Coordinate '{0}' not found in dataset")]
    CoordinateNotFound(String),
    #[error("Coordinate '{0}' not available for variable '{1}'")]
    CoordinateNotAvailableForVariable(String, String),
    #[error("Invalid coordinate range: {0}")]
    InvalidCoordinateRange(String),
    #[error("Index {0} out of bounds for coordinate '{1}' (size: {2})")]
    IndexOutOfBounds(usize, String, u32),
    #[error("URL generation error: {0}")]
    UrlGenerationError(String),
    #[error("No variables selected for query")]
    NoVariablesSelected,
}

/// High-level query builder for OpenDAP datasets
pub struct DatasetQuery<'a> {
    dataset: &'a DdsDataset,
    base_url: String,
    selected_variables: Vec<String>,
    coordinate_constraints: HashMap<String, CoordinateConstraint>,
}

impl<'a> DatasetQuery<'a> {
    /// Create a new query builder
    pub fn new(dataset: &'a DdsDataset, base_url: String) -> Self {
        Self {
            dataset,
            base_url,
            selected_variables: Vec::new(),
            coordinate_constraints: HashMap::new(),
        }
    }

    /// Select a single variable with validation
    pub fn select_variable(mut self, name: &str) -> Result<Self, QueryError> {
        if !self.dataset.has_variable(name) {
            return Err(QueryError::VariableNotFound(name.to_string()));
        }

        if !self.selected_variables.contains(&name.to_string()) {
            self.selected_variables.push(name.to_string());
        }

        Ok(self)
    }

    /// Select multiple variables with validation
    pub fn select_variables(mut self, names: &[&str]) -> Result<Self, QueryError> {
        for name in names {
            self = self.select_variable(name)?;
        }
        Ok(self)
    }

    /// Apply a coordinate constraint with validation
    pub fn select_by_coordinate(
        mut self,
        coord_name: &str,
        constraint: CoordinateConstraint,
    ) -> Result<Self, QueryError> {
        // Check if coordinate exists in dataset
        if !self.dataset.has_coordinate(coord_name) {
            return Err(QueryError::CoordinateNotFound(coord_name.to_string()));
        }

        // Validate constraint against coordinate size
        if let Some(coord_info) = self.dataset.get_coordinate_info(coord_name) {
            constraint.validate(coord_name, coord_info.size)?;
        }

        // Check if coordinate is available for selected variables
        if !self.selected_variables.is_empty() {
            for var_name in &self.selected_variables {
                if let Some(var_info) = self.dataset.get_variable_info(var_name) {
                    if !var_info.coordinates.contains(&coord_name.to_string()) {
                        return Err(QueryError::CoordinateNotAvailableForVariable(
                            coord_name.to_string(),
                            var_name.to_string(),
                        ));
                    }
                }
            }
        }

        self.coordinate_constraints
            .insert(coord_name.to_string(), constraint);
        Ok(self)
    }

    /// Generate a DODS URL with constraints
    pub fn dods_url(self) -> Result<String, QueryError> {
        if self.selected_variables.is_empty() {
            return Err(QueryError::NoVariablesSelected);
        }

        let mut url_builder = UrlBuilder::new(&self.base_url);

        // Add variables
        for var_name in &self.selected_variables {
            url_builder = url_builder.add_variable(var_name);
        }

        // Add coordinate constraints for each selected variable
        for var_name in &self.selected_variables {
            if let Some(var_info) = self.dataset.get_variable_info(var_name) {
                let mut constraint_indices = Vec::new();

                // Build constraint indices in coordinate order
                for coord_name in &var_info.coordinates {
                    if let Some(constraint) = self.coordinate_constraints.get(coord_name) {
                        constraint_indices.extend(constraint.to_index_ranges());
                    }
                }

                // Apply constraints if any exist
                if !constraint_indices.is_empty() {
                    url_builder =
                        url_builder.add_multidimensional_constraint(var_name, constraint_indices);
                }
            }
        }

        url_builder
            .dods_url()
            .map_err(|e| QueryError::UrlGenerationError(e.to_string()))
    }

    /// Generate a DAS URL
    pub fn das_url(&self) -> String {
        UrlBuilder::new(&self.base_url).das_url()
    }

    /// Generate a DDS URL
    pub fn dds_url(&self) -> String {
        UrlBuilder::new(&self.base_url).dds_url()
    }

    /// Validate the current query
    pub fn validate(&self) -> Result<(), QueryError> {
        if self.selected_variables.is_empty() {
            return Err(QueryError::NoVariablesSelected);
        }

        // Validate all coordinate constraints
        for (coord_name, constraint) in &self.coordinate_constraints {
            if let Some(coord_info) = self.dataset.get_coordinate_info(coord_name) {
                constraint.validate(coord_name, coord_info.size)?;
            }
        }

        Ok(())
    }

    /// Estimate the download size in bytes
    pub fn estimated_size(&self) -> usize {
        let mut total_size = 0;

        for var_name in &self.selected_variables {
            if let Some(var_info) = self.dataset.get_variable_info(var_name) {
                let mut var_size = var_info.data_type.byte_count();

                // Calculate size based on constraints
                for (coord_name, coord_size) in &var_info.dimensions {
                    let effective_size =
                        if let Some(constraint) = self.coordinate_constraints.get(coord_name) {
                            match constraint {
                                CoordinateConstraint::Single(_) => 1,
                                CoordinateConstraint::Indices { start, end, stride } => {
                                    let range_size = end - start + 1;
                                    if let Some(stride_val) = stride {
                                        range_size.div_ceil(*stride_val)
                                    } else {
                                        range_size
                                    }
                                }
                            }
                        } else {
                            *coord_size as usize
                        };
                    var_size *= effective_size;
                }

                total_size += var_size;
            }
        }

        total_size
    }

    /// Get the list of selected variables
    pub fn selected_variables(&self) -> &[String] {
        &self.selected_variables
    }

    /// Get the active coordinate constraints
    pub fn active_constraints(&self) -> &HashMap<String, CoordinateConstraint> {
        &self.coordinate_constraints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataset() -> DdsDataset {
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
    Grid {
     ARRAY:
        Float32 wind_speed[time = 100][latitude = 5][longitude = 10];
     MAPS:
        Int32 time[time = 100];
        Float32 latitude[latitude = 5];
        Float32 longitude[longitude = 10];
    } wind_speed;
} test_dataset;"#;

        DdsDataset::from_bytes(dds_content).unwrap()
    }

    #[test]
    fn test_coordinate_constraint_creation() {
        let single = CoordinateConstraint::single(5);
        assert_eq!(single, CoordinateConstraint::Single(5));

        let range = CoordinateConstraint::range(0, 10);
        assert_eq!(
            range,
            CoordinateConstraint::Indices {
                start: 0,
                end: 10,
                stride: None
            }
        );

        let range_with_stride = CoordinateConstraint::range_with_stride(0, 20, 2);
        assert_eq!(
            range_with_stride,
            CoordinateConstraint::Indices {
                start: 0,
                end: 20,
                stride: Some(2)
            }
        );

        let first = CoordinateConstraint::first();
        assert_eq!(first, CoordinateConstraint::Single(0));

        let last = CoordinateConstraint::last(100);
        assert_eq!(last, CoordinateConstraint::Single(99));
    }

    #[test]
    fn test_coordinate_constraint_validation() {
        let constraint = CoordinateConstraint::single(5);
        assert!(constraint.validate("test", 10).is_ok());
        assert!(constraint.validate("test", 5).is_err()); // Index 5 is out of bounds for size 5

        let range_constraint = CoordinateConstraint::range(0, 9);
        assert!(range_constraint.validate("test", 10).is_ok());
        assert!(range_constraint.validate("test", 5).is_err()); // End index 9 is out of bounds for size 5

        let invalid_range = CoordinateConstraint::range(10, 5);
        assert!(invalid_range.validate("test", 20).is_err()); // Start > end
    }

    #[test]
    fn test_basic_query_building() {
        let dataset = create_test_dataset();

        let query = dataset
            .query("https://example.com/data")
            .select_variable("temperature")
            .unwrap();

        assert_eq!(query.selected_variables(), &["temperature"]);

        let url = query.dods_url().unwrap();
        assert_eq!(url, "https://example.com/data.dods?temperature");
    }

    #[test]
    fn test_multiple_variable_selection() {
        let dataset = create_test_dataset();

        let query = dataset
            .query("https://example.com/data")
            .select_variables(&["temperature", "wind_speed"])
            .unwrap();

        assert_eq!(query.selected_variables(), &["temperature", "wind_speed"]);

        let url = query.dods_url().unwrap();
        assert_eq!(url, "https://example.com/data.dods?temperature,wind_speed");
    }

    #[test]
    fn test_coordinate_constraints() {
        let dataset = create_test_dataset();

        let query = dataset
            .query("https://example.com/data")
            .select_variable("temperature")
            .unwrap()
            .select_by_coordinate("time", CoordinateConstraint::range(0, 10))
            .unwrap()
            .select_by_coordinate("latitude", CoordinateConstraint::single(2))
            .unwrap()
            .select_by_coordinate(
                "longitude",
                CoordinateConstraint::range_with_stride(0, 8, 2),
            )
            .unwrap();

        let url = query.dods_url().unwrap();
        assert_eq!(
            url,
            "https://example.com/data.dods?temperature[0:10][2][0:2:8]"
        );
    }

    #[test]
    fn test_query_validation_errors() {
        let dataset = create_test_dataset();

        // Test variable not found
        let result = dataset
            .query("https://example.com/data")
            .select_variable("nonexistent");
        assert!(matches!(result, Err(QueryError::VariableNotFound(_))));

        // Test coordinate not found
        let result = dataset
            .query("https://example.com/data")
            .select_variable("temperature")
            .unwrap()
            .select_by_coordinate("nonexistent", CoordinateConstraint::single(0));
        assert!(matches!(result, Err(QueryError::CoordinateNotFound(_))));

        // Test coordinate not available for variable
        let result = dataset
            .query("https://example.com/data")
            .select_variable("latitude")
            .unwrap() // latitude array only has latitude coordinate
            .select_by_coordinate("time", CoordinateConstraint::single(0));
        assert!(matches!(
            result,
            Err(QueryError::CoordinateNotAvailableForVariable(_, _))
        ));

        // Test index out of bounds
        let result = dataset
            .query("https://example.com/data")
            .select_variable("temperature")
            .unwrap()
            .select_by_coordinate("latitude", CoordinateConstraint::single(10)); // latitude only has 5 elements
        assert!(matches!(result, Err(QueryError::IndexOutOfBounds(_, _, _))));

        // Test no variables selected
        let result = dataset.query("https://example.com/data").dods_url();
        assert!(matches!(result, Err(QueryError::NoVariablesSelected)));
    }

    #[test]
    fn test_estimated_size() {
        let dataset = create_test_dataset();

        // Full temperature variable: 100 * 5 * 10 * 4 bytes = 20000 bytes
        let query = dataset
            .query("https://example.com/data")
            .select_variable("temperature")
            .unwrap();
        assert_eq!(query.estimated_size(), 20000);

        // Subset: 11 * 1 * 5 * 4 bytes = 220 bytes
        let query = dataset
            .query("https://example.com/data")
            .select_variable("temperature")
            .unwrap()
            .select_by_coordinate("time", CoordinateConstraint::range(0, 10))
            .unwrap()
            .select_by_coordinate("latitude", CoordinateConstraint::single(2))
            .unwrap()
            .select_by_coordinate(
                "longitude",
                CoordinateConstraint::range_with_stride(0, 8, 2),
            )
            .unwrap();
        assert_eq!(query.estimated_size(), 220);
    }

    #[test]
    fn test_das_dds_urls() {
        let dataset = create_test_dataset();

        let query = dataset
            .query("https://example.com/data")
            .select_variable("temperature")
            .unwrap();

        assert_eq!(query.das_url(), "https://example.com/data.das");
        assert_eq!(query.dds_url(), "https://example.com/data.dds");
    }

    #[test]
    fn test_query_introspection() {
        let dataset = create_test_dataset();

        let query = dataset
            .query("https://example.com/data")
            .select_variable("temperature")
            .unwrap()
            .select_by_coordinate("time", CoordinateConstraint::range(0, 10))
            .unwrap();

        assert_eq!(query.selected_variables(), &["temperature"]);

        let constraints = query.active_constraints();
        assert_eq!(constraints.len(), 1);
        assert!(constraints.contains_key("time"));
        assert_eq!(constraints["time"], CoordinateConstraint::range(0, 10));

        assert!(query.validate().is_ok());
    }
}
