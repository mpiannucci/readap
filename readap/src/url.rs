use std::collections::HashMap;

/// Represents different types of coordinate selection for OpenDAP constraints
#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    /// Index-based selection (isel style) - [start, end] or single index
    Index(IndexSelection),
    /// Value-based selection (sel style) - for coordinate values
    Value(ValueSelection),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndexSelection {
    Single(usize),
    Range(usize, usize),         // [start, end] inclusive
    Stride(usize, usize, usize), // [start, stride, end]
    Multiple(Vec<usize>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueSelection {
    Single(f64),
    Range(f64, f64), // [min, max]
    Multiple(Vec<f64>),
    String(String), // for time/string coordinates
    StringRange(String, String),
    StringMultiple(Vec<String>),
}

/// Represents a constraint on a single variable with multiple dimensions
#[derive(Debug, Clone, PartialEq)]
pub struct VariableConstraint {
    pub name: String,
    pub dimensions: Vec<Selection>,
}

/// Builder for OpenDAP constraints supporting both index and value-based selection
#[derive(Debug, Clone, Default)]
pub struct ConstraintBuilder {
    constraints: Vec<VariableConstraint>,
}

impl ConstraintBuilder {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add index-based selection (isel style)
    pub fn isel(mut self, selections: HashMap<String, IndexSelection>) -> Self {
        for (var_name, selection) in selections {
            // Find existing constraint or create new one
            if let Some(constraint) = self.constraints.iter_mut().find(|c| c.name == var_name) {
                constraint.dimensions.push(Selection::Index(selection));
            } else {
                self.constraints.push(VariableConstraint {
                    name: var_name,
                    dimensions: vec![Selection::Index(selection)],
                });
            }
        }
        self
    }

    /// Add value-based selection (sel style) - requires coordinate lookup
    pub fn sel(mut self, selections: HashMap<String, ValueSelection>) -> Self {
        for (var_name, selection) in selections {
            if let Some(constraint) = self.constraints.iter_mut().find(|c| c.name == var_name) {
                constraint.dimensions.push(Selection::Value(selection));
            } else {
                self.constraints.push(VariableConstraint {
                    name: var_name,
                    dimensions: vec![Selection::Value(selection)],
                });
            }
        }
        self
    }

    /// Build the final constraint string for OpenDAP URLs
    pub fn build(&self) -> String {
        if self.constraints.is_empty() {
            return String::new();
        }

        self.constraints
            .iter()
            .map(|constraint| format_variable_constraint(constraint))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Get constraints for processing (e.g., coordinate lookups)
    pub fn constraints(&self) -> &[VariableConstraint] {
        &self.constraints
    }
}

/// Format a single variable constraint for OpenDAP URL
fn format_variable_constraint(constraint: &VariableConstraint) -> String {
    let mut result = constraint.name.clone();

    for selection in &constraint.dimensions {
        match selection {
            Selection::Index(idx_sel) => {
                result.push_str(&format_index_selection(idx_sel));
            }
            Selection::Value(_) => {
                // Value selections need to be resolved to indices first
                // This is a placeholder - actual implementation will need coordinate lookup
                result.push_str("[VALUE_LOOKUP_NEEDED]");
            }
        }
    }

    result
}

fn format_index_selection(selection: &IndexSelection) -> String {
    match selection {
        IndexSelection::Single(idx) => format!("[{}]", idx),
        IndexSelection::Range(start, end) => format!("[{}:{}]", start, end),
        IndexSelection::Stride(start, stride, end) => format!("[{}:{}:{}]", start, stride, end),
        IndexSelection::Multiple(indices) => {
            // For multiple indices, we need multiple constraints
            indices
                .iter()
                .map(|idx| format!("[{}]", idx))
                .collect::<String>()
        }
    }
}

/// OpenDAP URL builder for constructing .das, .dds, and .dods endpoints
#[derive(Debug, Clone)]
pub struct OpenDAPUrlBuilder {
    base_url: String,
}

impl OpenDAPUrlBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut url = base_url.into();
        // Remove trailing .nc or other extensions if present
        if url.ends_with(".nc") {
            url = url[..url.len() - 3].to_string();
        }

        Self { base_url: url }
    }

    /// Build DAS (Data Attribute Structure) URL
    pub fn das_url(&self) -> String {
        format!("{}.das", self.base_url)
    }

    /// Build DDS (Data Descriptor Structure) URL  
    pub fn dds_url(&self) -> String {
        format!("{}.dds", self.base_url)
    }

    /// Build DODS (Data Object) URL with optional constraints
    pub fn dods_url(&self, constraints: Option<&str>) -> String {
        match constraints {
            Some(c) if !c.is_empty() => format!("{}.dods?{}", self.base_url, c),
            _ => format!("{}.dods", self.base_url),
        }
    }

    /// Build DODS URL with constraint builder
    pub fn dods_url_with_constraints(&self, builder: &ConstraintBuilder) -> String {
        let constraint_str = builder.build();
        self.dods_url(if constraint_str.is_empty() {
            None
        } else {
            Some(&constraint_str)
        })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Coordinate-aware constraint resolver that maps value selections to index selections
#[derive(Debug)]
pub struct CoordinateResolver {
    // Will be populated with coordinate data when available
    coordinate_cache: HashMap<String, Vec<f64>>,
}

impl CoordinateResolver {
    pub fn new() -> Self {
        Self {
            coordinate_cache: HashMap::new(),
        }
    }

    /// Add coordinate data for a variable
    pub fn add_coordinates(&mut self, var_name: String, coords: Vec<f64>) {
        self.coordinate_cache.insert(var_name, coords);
    }

    /// Resolve value-based selections to index-based selections using nearest neighbor
    pub fn resolve_constraints(
        &self,
        builder: &ConstraintBuilder,
    ) -> Result<ConstraintBuilder, String> {
        let mut resolved_builder = ConstraintBuilder::new();

        for constraint in builder.constraints() {
            let mut resolved_dimensions = Vec::new();

            for selection in &constraint.dimensions {
                let resolved_selection = match selection {
                    Selection::Index(idx_sel) => Selection::Index(idx_sel.clone()),
                    Selection::Value(val_sel) => {
                        // Look up coordinates for this variable/dimension
                        let coords =
                            self.coordinate_cache.get(&constraint.name).ok_or_else(|| {
                                format!("No coordinates found for variable: {}", constraint.name)
                            })?;

                        let resolved_idx = self.resolve_value_selection(val_sel, coords)?;
                        Selection::Index(resolved_idx)
                    }
                };
                resolved_dimensions.push(resolved_selection);
            }

            resolved_builder.constraints.push(VariableConstraint {
                name: constraint.name.clone(),
                dimensions: resolved_dimensions,
            });
        }

        Ok(resolved_builder)
    }

    /// Convert value selection to index selection using nearest neighbor lookup
    fn resolve_value_selection(
        &self,
        selection: &ValueSelection,
        coords: &[f64],
    ) -> Result<IndexSelection, String> {
        match selection {
            ValueSelection::Single(value) => {
                let idx = find_nearest_index(coords, *value)?;
                Ok(IndexSelection::Single(idx))
            }
            ValueSelection::Range(min, max) => {
                let start_idx = find_nearest_index(coords, *min)?;
                let end_idx = find_nearest_index(coords, *max)?;
                Ok(IndexSelection::Range(
                    start_idx.min(end_idx),
                    start_idx.max(end_idx),
                ))
            }
            ValueSelection::Multiple(values) => {
                let indices: Result<Vec<_>, _> = values
                    .iter()
                    .map(|v| find_nearest_index(coords, *v))
                    .collect();
                Ok(IndexSelection::Multiple(indices?))
            }
            ValueSelection::String(_)
            | ValueSelection::StringRange(_, _)
            | ValueSelection::StringMultiple(_) => {
                Err("String coordinate lookup not yet implemented".to_string())
            }
        }
    }
}

/// Find the nearest index for a given coordinate value using binary search
pub fn find_nearest_index(coords: &[f64], target: f64) -> Result<usize, String> {
    if coords.is_empty() {
        return Err("Empty coordinate array".to_string());
    }

    // Handle edge cases
    if target <= coords[0] {
        return Ok(0);
    }
    if target >= coords[coords.len() - 1] {
        return Ok(coords.len() - 1);
    }

    // Binary search for nearest value
    let mut left = 0;
    let mut right = coords.len() - 1;

    while left < right {
        let mid = (left + right) / 2;
        if coords[mid] < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    // Check which is closer: left or left-1
    if left > 0 {
        let left_dist = (coords[left] - target).abs();
        let prev_dist = (coords[left - 1] - target).abs();
        if prev_dist < left_dist {
            Ok(left - 1)
        } else {
            Ok(left)
        }
    } else {
        Ok(left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_builder() {
        let builder = OpenDAPUrlBuilder::new("http://example.com/data");

        assert_eq!(builder.das_url(), "http://example.com/data.das");
        assert_eq!(builder.dds_url(), "http://example.com/data.dds");
        assert_eq!(builder.dods_url(None), "http://example.com/data.dods");
        assert_eq!(
            builder.dods_url(Some("temp[0:10]")),
            "http://example.com/data.dods?temp[0:10]"
        );
    }

    #[test]
    fn test_url_builder_with_extension() {
        let builder = OpenDAPUrlBuilder::new("http://example.com/data.nc");

        assert_eq!(builder.das_url(), "http://example.com/data.das");
        assert_eq!(builder.dds_url(), "http://example.com/data.dds");
        assert_eq!(builder.dods_url(None), "http://example.com/data.dods");
    }

    #[test]
    fn test_constraint_builder_isel() {
        let mut selections = HashMap::new();
        selections.insert("temperature".to_string(), IndexSelection::Range(0, 10));
        selections.insert("pressure".to_string(), IndexSelection::Single(5));

        let builder = ConstraintBuilder::new().isel(selections);
        let constraint_str = builder.build();

        // Should contain both variables with their constraints
        assert!(constraint_str.contains("temperature[0:10]"));
        assert!(constraint_str.contains("pressure[5]"));
    }

    #[test]
    fn test_nearest_neighbor_lookup() {
        let coords = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(find_nearest_index(&coords, 0.0).unwrap(), 0);
        assert_eq!(find_nearest_index(&coords, 2.4).unwrap(), 2);
        assert_eq!(find_nearest_index(&coords, 2.6).unwrap(), 3);
        assert_eq!(find_nearest_index(&coords, 5.0).unwrap(), 5);
        assert_eq!(find_nearest_index(&coords, 10.0).unwrap(), 5); // Beyond range
    }

    #[test]
    fn test_coordinate_resolver() {
        let mut resolver = CoordinateResolver::new();
        resolver.add_coordinates("time".to_string(), vec![0.0, 1.0, 2.0, 3.0, 4.0]);

        let mut selections = HashMap::new();
        selections.insert("time".to_string(), ValueSelection::Single(2.3));

        let value_builder = ConstraintBuilder::new().sel(selections);
        let resolved = resolver.resolve_constraints(&value_builder).unwrap();

        // 2.3 should resolve to index 2 (nearest to 2.0)
        let constraint_str = resolved.build();
        assert!(constraint_str.contains("time[2]"));
    }
}
