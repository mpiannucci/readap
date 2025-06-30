use crate::{ConstraintBuilder, CoordinateResolver, OpenDAPUrlBuilder, UniversalFetch};
use js_sys::{
    Array, Float32Array, Float64Array, Int16Array, Int32Array, Int8Array, Object, Reflect,
    Uint16Array, Uint32Array, Uint8Array,
};
use readap::{data::DataArray, parse_das_attributes, DdsDataset, DodsDataset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Information about a variable in the dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VariableInfo {
    name: String,
    data_type: String,
    dimensions: Vec<String>,
    attributes: HashMap<String, String>,
}

/// High-level OpenDAP dataset interface with xarray-style selection and automatic data fetching
#[wasm_bindgen]
pub struct OpenDAPDataset {
    url_builder: OpenDAPUrlBuilder,
    das_data: Option<String>,
    dds_data: Option<String>,
    coordinate_resolver: CoordinateResolver,
    variables: HashMap<String, VariableInfo>,
    coordinate_cache: HashMap<String, Array>, // Cache coordinate data as JS arrays
    fetch_client: UniversalFetch,             // Runtime-agnostic fetch client
}

#[wasm_bindgen]
impl OpenDAPDataset {
    /// Create a dataset from a base URL and automatically load metadata
    #[wasm_bindgen(js_name = fromURL)]
    pub async fn from_url(base_url: &str) -> Result<OpenDAPDataset, JsValue> {
        let url_builder = OpenDAPUrlBuilder::new(base_url);
        let coordinate_resolver = CoordinateResolver::new();
        let fetch_client = UniversalFetch::new()?;

        let mut dataset = OpenDAPDataset {
            url_builder,
            das_data: None,
            dds_data: None,
            coordinate_resolver,
            variables: HashMap::new(),
            coordinate_cache: HashMap::new(),
            fetch_client,
        };

        // Automatically load metadata
        dataset.load_metadata().await?;

        Ok(dataset)
    }

    /// Create a dataset from a base URL without loading metadata (for manual control)
    #[wasm_bindgen(js_name = fromURLLazy)]
    pub fn from_url_lazy(base_url: &str) -> Result<OpenDAPDataset, JsValue> {
        let url_builder = OpenDAPUrlBuilder::new(base_url);
        let coordinate_resolver = CoordinateResolver::new();
        let fetch_client = UniversalFetch::new()?;

        Ok(OpenDAPDataset {
            url_builder,
            das_data: None,
            dds_data: None,
            coordinate_resolver,
            variables: HashMap::new(),
            coordinate_cache: HashMap::new(),
            fetch_client,
        })
    }

    /// Create a dataset from DAS data
    #[wasm_bindgen(js_name = fromDAS)]
    pub fn from_das(das_data: &str) -> Result<OpenDAPDataset, JsValue> {
        let url_builder = OpenDAPUrlBuilder::new(""); // No URL needed for DAS-only
        let coordinate_resolver = CoordinateResolver::new();
        let fetch_client = UniversalFetch::new()?;

        let mut dataset = OpenDAPDataset {
            url_builder,
            das_data: Some(das_data.to_string()),
            dds_data: None,
            coordinate_resolver,
            variables: HashMap::new(),
            coordinate_cache: HashMap::new(),
            fetch_client,
        };

        dataset.parse_das()?;

        Ok(dataset)
    }

    /// Create a dataset from DDS data
    #[wasm_bindgen(js_name = fromDDS)]
    pub fn from_dds(dds_data: &str) -> Result<OpenDAPDataset, JsValue> {
        let url_builder = OpenDAPUrlBuilder::new(""); // No URL needed for DDS-only
        let coordinate_resolver = CoordinateResolver::new();
        let fetch_client = UniversalFetch::new()?;

        let mut dataset = OpenDAPDataset {
            url_builder,
            das_data: None,
            dds_data: Some(dds_data.to_string()),
            coordinate_resolver,
            variables: HashMap::new(),
            coordinate_cache: HashMap::new(),
            fetch_client,
        };

        dataset.parse_dds()?;

        Ok(dataset)
    }

    /// Parse DAS data (call after fetching DAS externally)
    #[wasm_bindgen(js_name = parseDAS)]
    pub fn parse_das_data(&mut self, das_data: &str) -> Result<(), JsValue> {
        self.das_data = Some(das_data.to_string());
        self.parse_das()
    }

    /// Parse DDS data (call after fetching DDS externally)
    #[wasm_bindgen(js_name = parseDDS)]
    pub fn parse_dds_data(&mut self, dds_data: &str) -> Result<(), JsValue> {
        self.dds_data = Some(dds_data.to_string());
        self.parse_dds()
    }

    /// Parse DODS binary data and return parsed variable data
    #[wasm_bindgen(js_name = parseDODS)]
    pub fn parse_dods_data(&self, dods_data: &Uint8Array) -> Result<Object, JsValue> {
        let data_vec = dods_data.to_vec();
        let dods_dataset = DodsDataset::from_bytes(&data_vec)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse DODS data: {:?}", e)))?;

        let result = Object::new();

        // Get all variable names and process each one
        let var_names = dods_dataset.variables();

        for var_name in var_names {
            if let Ok(data_array) = dods_dataset.variable_data(&var_name) {
                let typed_array = self.convert_data_array_to_typed_array(&data_array)?;
                let var_obj = Object::new();

                Reflect::set(
                    &var_obj,
                    &JsValue::from_str("name"),
                    &JsValue::from_str(&var_name),
                )?;
                Reflect::set(
                    &var_obj,
                    &JsValue::from_str("data"),
                    &typed_array.get_array(),
                )?;
                Reflect::set(
                    &var_obj,
                    &JsValue::from_str("type"),
                    &JsValue::from_str(&typed_array.get_type()),
                )?;
                Reflect::set(
                    &var_obj,
                    &JsValue::from_str("length"),
                    &JsValue::from_f64(typed_array.length() as f64),
                )?;

                Reflect::set(&result, &JsValue::from_str(&var_name), &var_obj)?;
            }
        }

        Ok(result)
    }

    /// Get variable names
    #[wasm_bindgen(js_name = getVariableNames)]
    pub fn get_variable_names(&self) -> Array {
        let names = Array::new();
        for name in self.variables.keys() {
            names.push(&JsValue::from_str(name));
        }
        names
    }

    /// Get variable information as JSON
    #[wasm_bindgen(js_name = getVariableInfo)]
    pub fn get_variable_info(&self, name: &str) -> Result<String, JsValue> {
        let info = self
            .variables
            .get(name)
            .ok_or_else(|| JsValue::from_str(&format!("Variable '{}' not found", name)))?;

        serde_json::to_string(info).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get all variables information as JSON
    #[wasm_bindgen(js_name = getVariablesInfo)]
    pub fn get_variables_info(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.variables).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Index-based selection (isel) - returns a constraint builder
    #[wasm_bindgen]
    pub fn isel(&self, selections: &JsValue) -> Result<DatasetSelection, JsValue> {
        let constraint_builder = ConstraintBuilder::new().isel(selections)?;
        Ok(DatasetSelection::new(constraint_builder))
    }

    /// Value-based selection (sel) - returns a constraint builder
    #[wasm_bindgen]
    pub fn sel(&self, selections: &JsValue) -> Result<DatasetSelection, JsValue> {
        let constraint_builder = ConstraintBuilder::new().sel(selections)?;
        Ok(DatasetSelection::new(constraint_builder))
    }

    /// Get variable data with automatic fetching and constraint resolution
    #[wasm_bindgen(js_name = getVariable)]
    pub async fn get_variable(
        &mut self,
        var_name: &str,
        constraints: Option<DatasetSelection>,
    ) -> Result<Object, JsValue> {
        // Build constraint string
        let constraint_str = match constraints {
            Some(selection) => {
                // Ensure coordinates are loaded for sel operations
                self.load_coordinates_for_selection(&selection).await?;

                // Resolve value-based constraints to index-based
                let resolved = self
                    .coordinate_resolver
                    .resolve_constraints(&selection.builder)?;
                resolved.build()
            }
            None => String::new(),
        };

        // Fetch DODS data
        let dods_url = if constraint_str.is_empty() {
            self.url_builder.dods_url(None)
        } else {
            self.url_builder.dods_url(Some(constraint_str))
        };

        let dods_data = self.fetch_client.fetch_binary(&dods_url).await?;

        // Parse and extract the specific variable
        let dods_dataset = DodsDataset::from_bytes(&dods_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse DODS data: {:?}", e)))?;

        if let Ok(data_array) = dods_dataset.variable_data(var_name) {
            let typed_array = self.convert_data_array_to_typed_array(&data_array)?;
            let var_obj = Object::new();

            Reflect::set(
                &var_obj,
                &JsValue::from_str("name"),
                &JsValue::from_str(var_name),
            )?;
            Reflect::set(
                &var_obj,
                &JsValue::from_str("data"),
                &typed_array.get_array(),
            )?;
            Reflect::set(
                &var_obj,
                &JsValue::from_str("type"),
                &JsValue::from_str(&typed_array.get_type()),
            )?;
            Reflect::set(
                &var_obj,
                &JsValue::from_str("length"),
                &JsValue::from_f64(typed_array.length() as f64),
            )?;

            Ok(var_obj)
        } else {
            Err(JsValue::from_str(&format!(
                "Variable '{}' not found in DODS data",
                var_name
            )))
        }
    }

    /// Get multiple variables with automatic fetching
    #[wasm_bindgen(js_name = getVariables)]
    pub async fn get_variables(
        &mut self,
        var_names: &Array,
        constraints: Option<DatasetSelection>,
    ) -> Result<Object, JsValue> {
        // Build constraint string with all requested variables
        let constraint_str = match constraints {
            Some(selection) => {
                // Ensure coordinates are loaded for sel operations
                self.load_coordinates_for_selection(&selection).await?;

                // Resolve value-based constraints to index-based
                let resolved = self
                    .coordinate_resolver
                    .resolve_constraints(&selection.builder)?;
                resolved.build()
            }
            None => {
                // Build constraint for all requested variables
                let mut var_constraints = Vec::new();
                for i in 0..var_names.length() {
                    if let Some(var_name) = var_names.get(i).as_string() {
                        var_constraints.push(var_name);
                    }
                }
                var_constraints.join(",")
            }
        };

        // Fetch DODS data
        let dods_url = if constraint_str.is_empty() {
            self.url_builder.dods_url(None)
        } else {
            self.url_builder.dods_url(Some(constraint_str))
        };

        let dods_data = self.fetch_client.fetch_binary(&dods_url).await?;

        // Parse and extract all requested variables
        let dods_dataset = DodsDataset::from_bytes(&dods_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse DODS data: {:?}", e)))?;

        let result = Object::new();

        for i in 0..var_names.length() {
            if let Some(var_name) = var_names.get(i).as_string() {
                if let Ok(data_array) = dods_dataset.variable_data(&var_name) {
                    let typed_array = self.convert_data_array_to_typed_array(&data_array)?;
                    let var_obj = Object::new();

                    Reflect::set(
                        &var_obj,
                        &JsValue::from_str("name"),
                        &JsValue::from_str(&var_name),
                    )?;
                    Reflect::set(
                        &var_obj,
                        &JsValue::from_str("data"),
                        &typed_array.get_array(),
                    )?;
                    Reflect::set(
                        &var_obj,
                        &JsValue::from_str("type"),
                        &JsValue::from_str(&typed_array.get_type()),
                    )?;
                    Reflect::set(
                        &var_obj,
                        &JsValue::from_str("length"),
                        &JsValue::from_f64(typed_array.length() as f64),
                    )?;

                    Reflect::set(&result, &JsValue::from_str(&var_name), &var_obj)?;
                }
            }
        }

        Ok(result)
    }

    /// Load coordinate data for a variable automatically via fetch
    #[wasm_bindgen(js_name = loadCoordinates)]
    pub async fn load_coordinates(&mut self, var_name: &str) -> Result<(), JsValue> {
        // Check if already cached
        if self.coordinate_cache.contains_key(var_name) {
            return Ok(());
        }

        // Fetch coordinate data using a simple constraint
        let constraint_str = format!("{}", var_name);
        let dods_url = self.url_builder.dods_url(Some(constraint_str));
        let dods_data = self.fetch_client.fetch_binary(&dods_url).await?;

        // Parse and extract coordinate values
        let dods_dataset = DodsDataset::from_bytes(&dods_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse DODS data: {:?}", e)))?;

        if let Ok(data_array) = dods_dataset.variable_data(var_name) {
            let coord_values = self.extract_coordinate_values(&data_array)?;

            // Cache the coordinates
            self.coordinate_cache
                .insert(var_name.to_string(), coord_values.clone());

            // Add to coordinate resolver
            self.coordinate_resolver
                .add_coordinates_from_array(var_name, &coord_values)?;
        }

        Ok(())
    }

    /// Add coordinate data for a variable to enable value-based selection (manual)
    #[wasm_bindgen(js_name = addCoordinates)]
    pub fn add_coordinates(&mut self, var_name: &str, coords: &Array) -> Result<(), JsValue> {
        // Cache the coordinates
        self.coordinate_cache
            .insert(var_name.to_string(), coords.clone());

        // Add to coordinate resolver
        self.coordinate_resolver
            .add_coordinates_from_array(var_name, coords)
    }

    /// Resolve value-based constraints to index-based constraints
    #[wasm_bindgen(js_name = resolveConstraints)]
    pub fn resolve_constraints(
        &self,
        builder: &ConstraintBuilder,
    ) -> Result<ConstraintBuilder, JsValue> {
        self.coordinate_resolver.resolve_constraints(builder)
    }

    /// Get the base URL
    #[wasm_bindgen(js_name = baseUrl)]
    pub fn base_url(&self) -> String {
        self.url_builder.base_url()
    }

    /// Get DAS URL
    #[wasm_bindgen(js_name = dasUrl)]
    pub fn das_url(&self) -> String {
        self.url_builder.das_url()
    }

    /// Get DDS URL
    #[wasm_bindgen(js_name = ddsUrl)]
    pub fn dds_url(&self) -> String {
        self.url_builder.dds_url()
    }

    /// Get DODS URL with optional constraints
    #[wasm_bindgen(js_name = dodsUrl)]
    pub fn dods_url(&self, constraints: Option<String>) -> String {
        self.url_builder.dods_url(constraints)
    }

    /// Get DODS URL with constraint builder
    #[wasm_bindgen(js_name = dodsUrlWithConstraints)]
    pub fn dods_url_with_constraints(&self, builder: &ConstraintBuilder) -> String {
        let constraint_str = builder.inner.build();
        self.url_builder.dods_url(if constraint_str.is_empty() {
            None
        } else {
            Some(constraint_str)
        })
    }
}

impl OpenDAPDataset {
    /// Load metadata (DAS and DDS) automatically via fetch
    async fn load_metadata(&mut self) -> Result<(), JsValue> {
        // Load DAS
        let das_url = self.url_builder.das_url();
        let das_response = self.fetch_client.fetch_text(&das_url).await?;
        self.das_data = Some(das_response);
        self.parse_das()?;

        // Load DDS
        let dds_url = self.url_builder.dds_url();
        let dds_response = self.fetch_client.fetch_text(&dds_url).await?;
        self.dds_data = Some(dds_response);
        self.parse_dds()?;

        Ok(())
    }

    /// Load coordinates needed for a selection operation
    async fn load_coordinates_for_selection(
        &mut self,
        _selection: &DatasetSelection,
    ) -> Result<(), JsValue> {
        // Extract coordinate variables that need to be loaded from the selection
        // This is a simplified implementation - in a full implementation, you'd analyze
        // the constraints to determine which coordinates are needed for value-based selections

        // For now, we'll assume common coordinate names
        let potential_coords = [
            "time",
            "lat",
            "latitude",
            "lon",
            "longitude",
            "depth",
            "level",
            "x",
            "y",
            "z",
        ];

        for coord_name in potential_coords.iter() {
            if self.variables.contains_key(*coord_name)
                && !self.coordinate_cache.contains_key(*coord_name)
            {
                // Try to load this coordinate
                if let Ok(()) = self.load_coordinates(coord_name).await {
                    // Successfully loaded
                }
            }
        }

        Ok(())
    }

    /// Extract coordinate values from DataArray for coordinate resolver
    fn extract_coordinate_values(&self, data_array: &DataArray) -> Result<Array, JsValue> {
        match data_array {
            DataArray::Float64(values) => {
                let result = Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    result.set(i as u32, JsValue::from_f64(val));
                }
                Ok(result)
            }
            DataArray::Float32(values) => {
                let result = Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    result.set(i as u32, JsValue::from_f64(val as f64));
                }
                Ok(result)
            }
            DataArray::Int32(values) => {
                let result = Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    result.set(i as u32, JsValue::from_f64(val as f64));
                }
                Ok(result)
            }
            DataArray::Int16(values) => {
                let result = Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    result.set(i as u32, JsValue::from_f64(val as f64));
                }
                Ok(result)
            }
            DataArray::UInt16(values) => {
                let result = Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    result.set(i as u32, JsValue::from_f64(val as f64));
                }
                Ok(result)
            }
            DataArray::UInt32(values) => {
                let result = Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    result.set(i as u32, JsValue::from_f64(val as f64));
                }
                Ok(result)
            }
            DataArray::Byte(values) => {
                let result = Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    result.set(i as u32, JsValue::from_f64(val as f64));
                }
                Ok(result)
            }
            _ => Err(JsValue::from_str("Coordinate data must be numeric")),
        }
    }

    fn parse_das(&mut self) -> Result<(), JsValue> {
        if let Some(das_text) = &self.das_data {
            let _das_attrs = parse_das_attributes(das_text)
                .map_err(|e| JsValue::from_str(&format!("Failed to parse DAS: {:?}", e)))?;
            // TODO: Extract variable attributes and merge with DDS info
        }
        Ok(())
    }

    fn parse_dds(&mut self) -> Result<(), JsValue> {
        if let Some(dds_text) = &self.dds_data {
            let dds_dataset = DdsDataset::from_bytes(dds_text)
                .map_err(|e| JsValue::from_str(&format!("Failed to parse DDS: {:?}", e)))?;

            // Extract variable information
            for value in &dds_dataset.values {
                let (name, data_type, dimensions) = match value {
                    readap::DdsValue::Array(arr) => (
                        arr.name.clone(),
                        format!("{:?}", arr.data_type),
                        arr.coords
                            .iter()
                            .map(|(name, _size)| name.clone())
                            .collect(),
                    ),
                    readap::DdsValue::Grid(grid) => (
                        grid.name.clone(),
                        format!("{:?}", grid.array.data_type),
                        grid.array
                            .coords
                            .iter()
                            .map(|(name, _size)| name.clone())
                            .collect(),
                    ),
                    readap::DdsValue::Structure(structure) => {
                        (structure.name.clone(), "Structure".to_string(), Vec::new())
                    }
                    readap::DdsValue::Sequence(sequence) => {
                        (sequence.name.clone(), "Sequence".to_string(), Vec::new())
                    }
                };

                let var_info = VariableInfo {
                    name: name.clone(),
                    data_type,
                    dimensions,
                    attributes: HashMap::new(), // TODO: merge with DAS attributes
                };
                self.variables.insert(name, var_info);
            }
        }
        Ok(())
    }

    fn convert_data_array_to_typed_array(
        &self,
        data_array: &DataArray,
    ) -> Result<TypedDataArray, JsValue> {
        match data_array {
            DataArray::Byte(values) => {
                let array = Int8Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    array.set_index(i as u32, val);
                }
                Ok(TypedDataArray::Int8(array))
            }
            DataArray::Int16(values) => {
                let array = Int16Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    array.set_index(i as u32, val);
                }
                Ok(TypedDataArray::Int16(array))
            }
            DataArray::UInt16(values) => {
                let array = Uint16Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    array.set_index(i as u32, val);
                }
                Ok(TypedDataArray::Uint16(array))
            }
            DataArray::Int32(values) => {
                let array = Int32Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    array.set_index(i as u32, val);
                }
                Ok(TypedDataArray::Int32(array))
            }
            DataArray::UInt32(values) => {
                let array = Uint32Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    array.set_index(i as u32, val);
                }
                Ok(TypedDataArray::Uint32(array))
            }
            DataArray::Float32(values) => {
                let array = Float32Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    array.set_index(i as u32, val);
                }
                Ok(TypedDataArray::Float32(array))
            }
            DataArray::Float64(values) => {
                let array = Float64Array::new_with_length(values.len() as u32);
                for (i, &val) in values.iter().enumerate() {
                    array.set_index(i as u32, val);
                }
                Ok(TypedDataArray::Float64(array))
            }
            DataArray::String(values) => {
                let array = Array::new_with_length(values.len() as u32);
                for (i, val) in values.iter().enumerate() {
                    array.set(i as u32, JsValue::from_str(val));
                }
                Ok(TypedDataArray::String(array))
            }
            DataArray::URL(values) => {
                let array = Array::new_with_length(values.len() as u32);
                for (i, val) in values.iter().enumerate() {
                    array.set(i as u32, JsValue::from_str(val));
                }
                Ok(TypedDataArray::String(array))
            }
        }
    }
}

/// Represents a dataset selection that can be chained and provides constraint building
#[wasm_bindgen]
pub struct DatasetSelection {
    pub(crate) builder: ConstraintBuilder,
}

impl DatasetSelection {
    pub(crate) fn new(builder: ConstraintBuilder) -> Self {
        Self { builder }
    }
}

#[wasm_bindgen]
impl DatasetSelection {
    /// Chain additional index-based selection
    #[wasm_bindgen]
    pub fn isel(mut self, selections: &JsValue) -> Result<DatasetSelection, JsValue> {
        self.builder = self.builder.isel(selections)?;
        Ok(self)
    }

    /// Chain additional value-based selection
    #[wasm_bindgen]
    pub fn sel(mut self, selections: &JsValue) -> Result<DatasetSelection, JsValue> {
        self.builder = self.builder.sel(selections)?;
        Ok(self)
    }

    /// Get the constraint string for debugging
    #[wasm_bindgen(js_name = getConstraints)]
    pub fn get_constraints(&self) -> String {
        self.builder.build()
    }

    /// Get the underlying constraint builder
    #[wasm_bindgen(js_name = getBuilder)]
    pub fn get_builder(&self) -> ConstraintBuilder {
        self.builder.clone()
    }
}

/// Wrapper for different typed arrays returned by the dataset parsing
pub struct TypedDataArray {
    inner: TypedArrayInner,
}

enum TypedArrayInner {
    Int8(Int8Array),
    Uint8(Uint8Array),
    Int16(Int16Array),
    Uint16(Uint16Array),
    Int32(Int32Array),
    Uint32(Uint32Array),
    Float32(Float32Array),
    Float64(Float64Array),
    String(Array),
}

impl TypedDataArray {
    fn Int8(array: Int8Array) -> Self {
        Self {
            inner: TypedArrayInner::Int8(array),
        }
    }

    fn Uint8(array: Uint8Array) -> Self {
        Self {
            inner: TypedArrayInner::Uint8(array),
        }
    }

    fn Int16(array: Int16Array) -> Self {
        Self {
            inner: TypedArrayInner::Int16(array),
        }
    }

    fn Uint16(array: Uint16Array) -> Self {
        Self {
            inner: TypedArrayInner::Uint16(array),
        }
    }

    fn Int32(array: Int32Array) -> Self {
        Self {
            inner: TypedArrayInner::Int32(array),
        }
    }

    fn Uint32(array: Uint32Array) -> Self {
        Self {
            inner: TypedArrayInner::Uint32(array),
        }
    }

    fn Float32(array: Float32Array) -> Self {
        Self {
            inner: TypedArrayInner::Float32(array),
        }
    }

    fn Float64(array: Float64Array) -> Self {
        Self {
            inner: TypedArrayInner::Float64(array),
        }
    }

    fn String(array: Array) -> Self {
        Self {
            inner: TypedArrayInner::String(array),
        }
    }

    /// Get the type name as a string
    pub fn get_type(&self) -> String {
        match &self.inner {
            TypedArrayInner::Int8(_) => "Int8Array".to_string(),
            TypedArrayInner::Uint8(_) => "Uint8Array".to_string(),
            TypedArrayInner::Int16(_) => "Int16Array".to_string(),
            TypedArrayInner::Uint16(_) => "Uint16Array".to_string(),
            TypedArrayInner::Int32(_) => "Int32Array".to_string(),
            TypedArrayInner::Uint32(_) => "Uint32Array".to_string(),
            TypedArrayInner::Float32(_) => "Float32Array".to_string(),
            TypedArrayInner::Float64(_) => "Float64Array".to_string(),
            TypedArrayInner::String(_) => "StringArray".to_string(),
        }
    }

    /// Get the length of the array
    pub fn length(&self) -> u32 {
        match &self.inner {
            TypedArrayInner::Int8(arr) => arr.length(),
            TypedArrayInner::Uint8(arr) => arr.length(),
            TypedArrayInner::Int16(arr) => arr.length(),
            TypedArrayInner::Uint16(arr) => arr.length(),
            TypedArrayInner::Int32(arr) => arr.length(),
            TypedArrayInner::Uint32(arr) => arr.length(),
            TypedArrayInner::Float32(arr) => arr.length(),
            TypedArrayInner::Float64(arr) => arr.length(),
            TypedArrayInner::String(arr) => arr.length(),
        }
    }

    /// Get the underlying JavaScript array/typed array  
    pub fn get_array(&self) -> JsValue {
        match &self.inner {
            TypedArrayInner::Int8(arr) => arr.clone().into(),
            TypedArrayInner::Uint8(arr) => arr.clone().into(),
            TypedArrayInner::Int16(arr) => arr.clone().into(),
            TypedArrayInner::Uint16(arr) => arr.clone().into(),
            TypedArrayInner::Int32(arr) => arr.clone().into(),
            TypedArrayInner::Uint32(arr) => arr.clone().into(),
            TypedArrayInner::Float32(arr) => arr.clone().into(),
            TypedArrayInner::Float64(arr) => arr.clone().into(),
            TypedArrayInner::String(arr) => arr.clone().into(),
        }
    }
}
