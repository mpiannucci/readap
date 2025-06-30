use readap::url::{ConstraintBuilder as CoreConstraintBuilder, IndexSelection, ValueSelection};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Simplified constraint builder with method chaining API
/// This avoids complex JavaScript object parsing that causes issues in Node.js/Bun
#[wasm_bindgen]
#[derive(Clone)]
pub struct SimpleConstraintBuilder {
    inner: CoreConstraintBuilder,
}

#[wasm_bindgen]
impl SimpleConstraintBuilder {
    /// Create a new constraint builder
    #[wasm_bindgen(constructor)]
    pub fn new() -> SimpleConstraintBuilder {
        SimpleConstraintBuilder {
            inner: CoreConstraintBuilder::new(),
        }
    }

    /// Add a single index constraint (e.g., time[5])
    #[wasm_bindgen(js_name = addSingle)]
    pub fn add_single(self, var_name: &str, index: usize) -> SimpleConstraintBuilder {
        let mut selections = HashMap::new();
        selections.insert(var_name.to_string(), IndexSelection::Single(index));

        SimpleConstraintBuilder {
            inner: self.inner.isel(selections),
        }
    }

    /// Add a range constraint (e.g., time[0:10])
    #[wasm_bindgen(js_name = addRange)]
    pub fn add_range(self, var_name: &str, start: usize, end: usize) -> SimpleConstraintBuilder {
        let mut selections = HashMap::new();
        selections.insert(var_name.to_string(), IndexSelection::Range(start, end));

        SimpleConstraintBuilder {
            inner: self.inner.isel(selections),
        }
    }

    /// Add a stride constraint (e.g., time[0:2:10])
    #[wasm_bindgen(js_name = addStride)]
    pub fn add_stride(
        self,
        var_name: &str,
        start: usize,
        stride: usize,
        end: usize,
    ) -> SimpleConstraintBuilder {
        let mut selections = HashMap::new();
        selections.insert(
            var_name.to_string(),
            IndexSelection::Stride(start, stride, end),
        );

        SimpleConstraintBuilder {
            inner: self.inner.isel(selections),
        }
    }

    /// Add multiple specific indices (e.g., time[0,5,10])
    #[wasm_bindgen(js_name = addMultiple)]
    pub fn add_multiple(self, var_name: &str, indices: &[usize]) -> SimpleConstraintBuilder {
        let mut selections = HashMap::new();
        selections.insert(
            var_name.to_string(),
            IndexSelection::Multiple(indices.to_vec()),
        );

        SimpleConstraintBuilder {
            inner: self.inner.isel(selections),
        }
    }

    /// Add a value-based single constraint for nearest neighbor lookup
    #[wasm_bindgen(js_name = addValueSingle)]
    pub fn add_value_single(self, var_name: &str, value: f64) -> SimpleConstraintBuilder {
        let mut selections = HashMap::new();
        selections.insert(var_name.to_string(), ValueSelection::Single(value));

        SimpleConstraintBuilder {
            inner: self.inner.sel(selections),
        }
    }

    /// Add a value-based range constraint
    #[wasm_bindgen(js_name = addValueRange)]
    pub fn add_value_range(self, var_name: &str, min: f64, max: f64) -> SimpleConstraintBuilder {
        let mut selections = HashMap::new();
        selections.insert(var_name.to_string(), ValueSelection::Range(min, max));

        SimpleConstraintBuilder {
            inner: self.inner.sel(selections),
        }
    }

    /// Add a string-based constraint (for time/date values)
    #[wasm_bindgen(js_name = addValueString)]
    pub fn add_value_string(self, var_name: &str, value: &str) -> SimpleConstraintBuilder {
        let mut selections = HashMap::new();
        selections.insert(
            var_name.to_string(),
            ValueSelection::String(value.to_string()),
        );

        SimpleConstraintBuilder {
            inner: self.inner.sel(selections),
        }
    }

    /// Add multiple value-based constraints
    #[wasm_bindgen(js_name = addValueMultiple)]
    pub fn add_value_multiple(self, var_name: &str, values: &[f64]) -> SimpleConstraintBuilder {
        let mut selections = HashMap::new();
        selections.insert(
            var_name.to_string(),
            ValueSelection::Multiple(values.to_vec()),
        );

        SimpleConstraintBuilder {
            inner: self.inner.sel(selections),
        }
    }

    /// Build the constraint string
    #[wasm_bindgen]
    pub fn build(&self) -> String {
        self.inner.build()
    }

    /// Reset the builder to start fresh
    #[wasm_bindgen]
    pub fn reset(self) -> SimpleConstraintBuilder {
        SimpleConstraintBuilder {
            inner: CoreConstraintBuilder::new(),
        }
    }

    /// Create a copy of the builder
    #[wasm_bindgen]
    pub fn clone(&self) -> SimpleConstraintBuilder {
        SimpleConstraintBuilder {
            inner: self.inner.clone(),
        }
    }

    /// Get debug information about the current constraints
    #[wasm_bindgen(js_name = getInfo)]
    pub fn get_info(&self) -> String {
        format!("Constraints: {}", self.inner.build())
    }
}

/// String-based constraint builder for maximum compatibility
/// This allows building constraints using simple string operations
#[wasm_bindgen]
pub struct StringConstraintBuilder {
    constraints: Vec<String>,
}

#[wasm_bindgen]
impl StringConstraintBuilder {
    /// Create a new string-based constraint builder
    #[wasm_bindgen(constructor)]
    pub fn new() -> StringConstraintBuilder {
        StringConstraintBuilder {
            constraints: Vec::new(),
        }
    }

    /// Add a constraint from a string (e.g., "time[0]", "lat[10:20]")
    #[wasm_bindgen(js_name = addConstraint)]
    pub fn add_constraint(mut self, constraint: &str) -> StringConstraintBuilder {
        self.constraints.push(constraint.to_string());
        self
    }

    /// Add a variable selection (e.g., "temperature")
    #[wasm_bindgen(js_name = addVariable)]
    pub fn add_variable(mut self, var_name: &str) -> StringConstraintBuilder {
        self.constraints.push(var_name.to_string());
        self
    }

    /// Build the complete constraint string
    #[wasm_bindgen]
    pub fn build(&self) -> String {
        self.constraints.join(",")
    }

    /// Reset the builder
    #[wasm_bindgen]
    pub fn reset(self) -> StringConstraintBuilder {
        StringConstraintBuilder {
            constraints: Vec::new(),
        }
    }

    /// Get the number of constraints
    #[wasm_bindgen(js_name = getCount)]
    pub fn get_count(&self) -> usize {
        self.constraints.len()
    }
}
