//! URL Builder for OpenDAP endpoints
//!
//! This module provides a builder pattern for constructing OpenDAP URLs with support for
//! DAS (Dataset Attribute Structure), DDS (Dataset Descriptor Structure), and DODS
//! (Dataset Data Structure) endpoints, including constraint expressions for subsetting data.
//!
//! # Examples
//!
//! ## Basic URL construction
//! ```
//! use readap::UrlBuilder;
//!
//! let builder = UrlBuilder::new("https://example.com/data/dataset");
//! assert_eq!(builder.das_url(), "https://example.com/data/dataset.das");
//! assert_eq!(builder.dds_url(), "https://example.com/data/dataset.dds");
//! assert_eq!(builder.dods_url().unwrap(), "https://example.com/data/dataset.dods");
//! ```
//!
//! ## Variable selection and constraints
//! ```
//! use readap::{UrlBuilder, IndexRange};
//!
//! let url = UrlBuilder::new("https://example.com/data/ocean")
//!     .add_variable("temperature")
//!     .add_multidimensional_constraint("temperature", vec![
//!         IndexRange::Range { start: 0, end: 10, stride: None },      // time
//!         IndexRange::Single(5),                                      // depth
//!         IndexRange::Range { start: 20, end: 50, stride: Some(2) },  // latitude
//!         IndexRange::Range { start: -180, end: 180, stride: None },  // longitude
//!     ])
//!     .dods_url()
//!     .unwrap();
//!
//! assert_eq!(url, "https://example.com/data/ocean.dods?temperature[0:10][5][20:2:50][-180:180]");
//! ```

use crate::errors::Error;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UrlBuilder {
    base_url: String,
    variables: Vec<String>,
    constraints: HashMap<String, Vec<Constraint>>,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub variable: String,
    pub indices: Vec<IndexRange>,
}

#[derive(Debug, Clone)]
pub enum IndexRange {
    Single(isize),
    Range {
        start: isize,
        end: isize,
        stride: Option<isize>,
    },
}

impl UrlBuilder {
    pub fn new<S: Into<String>>(base_url: S) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }

        Self {
            base_url: url,
            variables: Vec::new(),
            constraints: HashMap::new(),
        }
    }

    pub fn das_url(&self) -> String {
        format!("{}.das", self.base_url)
    }

    pub fn dds_url(&self) -> String {
        format!("{}.dds", self.base_url)
    }

    pub fn dods_url(&self) -> Result<String, Error> {
        let mut url = format!("{}.dods", self.base_url);

        if !self.variables.is_empty() || !self.constraints.is_empty() {
            url.push('?');

            let mut parts = Vec::new();

            // Process variables that were explicitly added
            for variable in &self.variables {
                if let Some(constraints) = self.constraints.get(variable) {
                    // Combine all constraints for this variable into a single expression
                    let mut combined_constraint = variable.clone();
                    for constraint in constraints {
                        for index_range in &constraint.indices {
                            combined_constraint.push_str(&format!("[{index_range}]"));
                        }
                    }
                    parts.push(combined_constraint);
                } else {
                    parts.push(variable.clone());
                }
            }

            // Process constraints for variables that weren't explicitly added
            for (variable, constraints) in &self.constraints {
                if !self.variables.contains(variable) {
                    for constraint in constraints {
                        parts.push(constraint.to_string());
                    }
                }
            }

            url.push_str(&parts.join(","));
        }

        Ok(url)
    }

    pub fn add_variable<S: Into<String>>(mut self, variable: S) -> Self {
        self.variables.push(variable.into());
        self
    }

    pub fn add_variables<S: Into<String>>(mut self, variables: Vec<S>) -> Self {
        for var in variables {
            self.variables.push(var.into());
        }
        self
    }

    pub fn add_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints
            .entry(constraint.variable.clone())
            .or_default()
            .push(constraint);
        self
    }

    pub fn add_index_constraint<S: Into<String>>(
        self,
        variable: S,
        indices: Vec<IndexRange>,
    ) -> Self {
        let constraint = Constraint {
            variable: variable.into(),
            indices,
        };
        self.add_constraint(constraint)
    }

    pub fn add_single_index<S: Into<String>>(self, variable: S, index: isize) -> Self {
        self.add_index_constraint(variable, vec![IndexRange::Single(index)])
    }

    pub fn add_range<S: Into<String>>(
        self,
        variable: S,
        start: isize,
        end: isize,
        stride: Option<isize>,
    ) -> Self {
        self.add_index_constraint(variable, vec![IndexRange::Range { start, end, stride }])
    }

    pub fn add_multidimensional_constraint<S: Into<String>>(
        self,
        variable: S,
        indices: Vec<IndexRange>,
    ) -> Self {
        self.add_index_constraint(variable, indices)
    }

    pub fn clear_variables(mut self) -> Self {
        self.variables.clear();
        self
    }

    pub fn clear_constraints(mut self) -> Self {
        self.constraints.clear();
        self
    }

    pub fn clear_all(mut self) -> Self {
        self.variables.clear();
        self.constraints.clear();
        self
    }
}

impl Constraint {
    pub fn new<S: Into<String>>(variable: S, indices: Vec<IndexRange>) -> Self {
        Self {
            variable: variable.into(),
            indices,
        }
    }

    pub fn single<S: Into<String>>(variable: S, index: isize) -> Self {
        Self::new(variable, vec![IndexRange::Single(index)])
    }

    pub fn range<S: Into<String>>(
        variable: S,
        start: isize,
        end: isize,
        stride: Option<isize>,
    ) -> Self {
        Self::new(variable, vec![IndexRange::Range { start, end, stride }])
    }
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.variable)?;
        for index_range in &self.indices {
            write!(f, "[{index_range}]")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for IndexRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexRange::Single(index) => write!(f, "{index}"),
            IndexRange::Range { start, end, stride } => {
                if let Some(stride) = stride {
                    write!(f, "{start}:{stride}:{end}")
                } else {
                    write!(f, "{start}:{end}")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_urls() {
        let builder = UrlBuilder::new("http://example.com/data/dataset");

        assert_eq!(builder.das_url(), "http://example.com/data/dataset.das");
        assert_eq!(builder.dds_url(), "http://example.com/data/dataset.dds");
        assert_eq!(
            builder.dods_url().unwrap(),
            "http://example.com/data/dataset.dods"
        );
    }

    #[test]
    fn test_trailing_slash_removal() {
        let builder = UrlBuilder::new("http://example.com/data/dataset/");

        assert_eq!(builder.das_url(), "http://example.com/data/dataset.das");
        assert_eq!(builder.dds_url(), "http://example.com/data/dataset.dds");
        assert_eq!(
            builder.dods_url().unwrap(),
            "http://example.com/data/dataset.dods"
        );
    }

    #[test]
    fn test_simple_variable_selection() {
        let builder = UrlBuilder::new("http://example.com/data/dataset")
            .add_variable("temperature")
            .add_variable("pressure");

        let url = builder.dods_url().unwrap();
        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature,pressure"
        );
    }

    #[test]
    fn test_single_index_constraint() {
        let builder = UrlBuilder::new("http://example.com/data/dataset")
            .add_variable("temperature")
            .add_single_index("temperature", 5);

        let url = builder.dods_url().unwrap();
        assert_eq!(url, "http://example.com/data/dataset.dods?temperature[5]");
    }

    #[test]
    fn test_range_constraint() {
        let builder = UrlBuilder::new("http://example.com/data/dataset")
            .add_variable("temperature")
            .add_range("temperature", 0, 10, None);

        let url = builder.dods_url().unwrap();
        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature[0:10]"
        );
    }

    #[test]
    fn test_range_with_stride() {
        let builder = UrlBuilder::new("http://example.com/data/dataset")
            .add_variable("temperature")
            .add_range("temperature", 0, 20, Some(2));

        let url = builder.dods_url().unwrap();
        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature[0:2:20]"
        );
    }

    #[test]
    fn test_multiple_dimensions() {
        let constraint = Constraint::new(
            "temperature",
            vec![
                IndexRange::Range {
                    start: 0,
                    end: 10,
                    stride: None,
                },
                IndexRange::Single(5),
                IndexRange::Range {
                    start: 20,
                    end: 30,
                    stride: Some(2),
                },
            ],
        );

        let builder = UrlBuilder::new("http://example.com/data/dataset")
            .add_variable("temperature")
            .add_constraint(constraint);

        let url = builder.dods_url().unwrap();
        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature[0:10][5][20:2:30]"
        );
    }

    #[test]
    fn test_multiple_variables_with_constraints() {
        let builder = UrlBuilder::new("http://example.com/data/dataset")
            .add_variable("temperature")
            .add_variable("pressure")
            .add_range("temperature", 0, 10, None)
            .add_single_index("pressure", 3);

        let url = builder.dods_url().unwrap();
        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature[0:10],pressure[3]"
        );
    }

    #[test]
    fn test_constraint_without_variable() {
        let builder = UrlBuilder::new("http://example.com/data/dataset").add_range(
            "temperature",
            0,
            10,
            None,
        );

        let url = builder.dods_url().unwrap();
        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature[0:10]"
        );
    }

    #[test]
    fn test_clear_operations() {
        let builder = UrlBuilder::new("http://example.com/data/dataset")
            .add_variable("temperature")
            .add_variable("pressure")
            .add_range("temperature", 0, 10, None)
            .clear_variables();

        let url = builder.dods_url().unwrap();
        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature[0:10]"
        );

        let builder = builder.clear_constraints();
        let url = builder.dods_url().unwrap();
        assert_eq!(url, "http://example.com/data/dataset.dods");
    }

    #[test]
    fn test_constraint_display() {
        let constraint = Constraint::single("temp", 5);
        assert_eq!(constraint.to_string(), "temp[5]");

        let constraint = Constraint::range("temp", 0, 10, None);
        assert_eq!(constraint.to_string(), "temp[0:10]");

        let constraint = Constraint::range("temp", 0, 20, Some(2));
        assert_eq!(constraint.to_string(), "temp[0:2:20]");
    }

    #[test]
    fn test_index_range_display() {
        let single = IndexRange::Single(5);
        assert_eq!(single.to_string(), "5");

        let range = IndexRange::Range {
            start: 0,
            end: 10,
            stride: None,
        };
        assert_eq!(range.to_string(), "0:10");

        let range_with_stride = IndexRange::Range {
            start: 0,
            end: 20,
            stride: Some(2),
        };
        assert_eq!(range_with_stride.to_string(), "0:2:20");
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let url = UrlBuilder::new("http://example.com/data/dataset")
            .add_variable("temperature")
            .add_variable("pressure")
            .add_range("temperature", 0, 10, None)
            .add_single_index("pressure", 5)
            .dods_url()
            .unwrap();

        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature[0:10],pressure[5]"
        );
    }

    #[test]
    fn test_add_variables_batch() {
        let builder = UrlBuilder::new("http://example.com/data/dataset").add_variables(vec![
            "temperature",
            "pressure",
            "humidity",
        ]);

        let url = builder.dods_url().unwrap();
        assert_eq!(
            url,
            "http://example.com/data/dataset.dods?temperature,pressure,humidity"
        );
    }
}
