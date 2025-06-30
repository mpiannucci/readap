/// XArray-style Dataset API with coordinate-based selection
/// This provides a high-level interface similar to xarray.Dataset
/// with coordinate downloading, indexing, and selection capabilities
use crate::{ImmutableDataset, SimpleConstraintBuilder};
use js_sys::{Array, Object, Reflect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Coordinate information with values and indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateInfo {
    pub name: String,
    pub values: Vec<f64>,
    pub size: usize,
    pub units: Option<String>,
    pub long_name: Option<String>,
}

impl CoordinateInfo {
    /// Find the index closest to a given coordinate value
    pub fn nearest_index(&self, value: f64) -> usize {
        let mut best_idx = 0;
        let mut best_distance = (self.values[0] - value).abs();
        
        for (i, &coord_val) in self.values.iter().enumerate() {
            let distance = (coord_val - value).abs();
            if distance < best_distance {
                best_distance = distance;
                best_idx = i;
            }
        }
        
        best_idx
    }
    
    /// Find indices for a range of coordinate values
    pub fn range_indices(&self, min_val: f64, max_val: f64) -> (usize, usize) {
        let min_idx = self.nearest_index(min_val);
        let max_idx = self.nearest_index(max_val);
        (min_idx.min(max_idx), min_idx.max(max_idx))
    }
}

/// Selection specification for coordinate-based subsetting
#[derive(Debug, Clone)]
pub struct Selection {
    pub coordinate_selections: HashMap<String, SelectionType>,
}

#[derive(Debug, Clone)]
pub enum SelectionType {
    Single(f64),           // Single coordinate value
    Range(f64, f64),       // Range of coordinate values
    Index(usize),          // Direct index selection
    IndexRange(usize, usize), // Direct index range
}

/// High-level XArray-style Dataset with coordinate indexing
#[wasm_bindgen]
pub struct XArrayDataset {
    dataset: ImmutableDataset,
    coordinates: HashMap<String, CoordinateInfo>,
    variable_names: Vec<String>,
    grid_variables: std::collections::HashSet<String>,
}

#[wasm_bindgen]
impl XArrayDataset {
    /// Create a new XArray-style dataset from a URL with coordinate preloading
    #[wasm_bindgen(js_name = fromURL)]
    pub async fn from_url(base_url: &str) -> Result<XArrayDataset, JsValue> {
        // Load the base dataset
        let dataset = ImmutableDataset::from_url(base_url).await?;
        
        // Get variable names
        let var_names = dataset.get_variable_names();
        let variable_names: Vec<String> = (0..var_names.length())
            .map(|i| var_names.get(i).as_string().unwrap_or_default())
            .collect();
        
        // Get DDS content to identify grid variables and coordinates
        let dds_url = dataset.dds_url();
        let fetch_client = crate::UniversalFetch::new()?;
        let dds_content = fetch_client.fetch_text(&dds_url).await?;
        
        // Parse DDS to identify coordinates and grid variables
        let (coordinate_vars, grid_vars) = Self::parse_dds_structure(&dds_content)?;
        
        // Download coordinate data
        let mut coordinates = HashMap::new();
        
        web_sys::console::log_1(&"Loading coordinate data...".into());
        
        for coord_name in &coordinate_vars {
            if variable_names.contains(coord_name) {
                match Self::load_coordinate(&dataset, coord_name).await {
                    Ok(coord_info) => {
                        web_sys::console::log_1(&format!("✓ Loaded coordinate '{}': {} points", coord_name, coord_info.size).into());
                        coordinates.insert(coord_name.clone(), coord_info);
                    }
                    Err(e) => {
                        web_sys::console::warn_1(&format!("⚠ Failed to load coordinate '{}': {:?}", coord_name, e).into());
                    }
                }
            }
        }
        
        web_sys::console::log_1(&format!("Dataset initialized with {} coordinates", coordinates.len()).into());
        
        Ok(XArrayDataset {
            dataset,
            coordinates,
            variable_names,
            grid_variables: grid_vars,
        })
    }
    
    /// Get coordinate information as JSON
    #[wasm_bindgen(js_name = getCoordinates)]
    pub fn get_coordinates(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.coordinates)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    /// Get variable names
    #[wasm_bindgen(js_name = getVariableNames)]
    pub fn get_variable_names(&self) -> Array {
        let names = Array::new();
        for name in &self.variable_names {
            names.push(&JsValue::from_str(name));
        }
        names
    }
    
    /// Select data using coordinate values (xarray-style .sel())
    #[wasm_bindgen(js_name = sel)]
    pub async fn sel(&self, variable: &str, selections: &Object) -> Result<Object, JsValue> {
        // Parse selections from JavaScript object
        let mut coord_selections = HashMap::new();
        
        for key in js_sys::Object::keys(selections) {
            let key_str = key.as_string().unwrap();
            let value = Reflect::get(selections, &key)?;
            
            // Handle different selection types
            if value.is_object() {
                // Range selection: {min: 40, max: 50}
                let min_val = Reflect::get(&value, &JsValue::from_str("min"))?;
                let max_val = Reflect::get(&value, &JsValue::from_str("max"))?;
                
                if min_val.is_truthy() && max_val.is_truthy() && min_val.as_f64().is_some() && max_val.as_f64().is_some() {
                    let min = min_val.as_f64().unwrap();
                    let max = max_val.as_f64().unwrap();
                    coord_selections.insert(key_str, SelectionType::Range(min, max));
                }
            } else if value.is_truthy() && value.as_f64().is_some() {
                // Single value selection
                let val = value.as_f64().unwrap();
                coord_selections.insert(key_str, SelectionType::Single(val));
            }
        }
        
        // Convert coordinate selections to index constraints
        let mut constraint_parts: Vec<String> = Vec::new();
        
        // Get variable info to understand its dimensions
        let variables_info = self.dataset.get_variables_info()?;
        let vars: HashMap<String, serde_json::Value> = serde_json::from_str(&variables_info)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let var_info = vars.get(variable)
            .ok_or_else(|| JsValue::from_str(&format!("Variable '{}' not found", variable)))?;
        
        // Check if this is a grid variable
        let is_grid_variable = self.grid_variables.contains(variable);
        
        // Build constraints based on coordinate selections
        if is_grid_variable {
            // For grid variables, use direct constraint format: var[dim1_range][dim2_range]
            let mut parts: Vec<String> = Vec::new();
            
            // We need to infer dimensions from coordinate selections
            // For now, assume standard order: [longitude][latitude][time][step]
            let standard_dims = ["longitude", "latitude", "time", "step"];
            
            for &dim_name in &standard_dims {
                if let Some(selection) = coord_selections.get(dim_name) {
                    if let Some(coord_info) = self.coordinates.get(dim_name) {
                        let range_str = match selection {
                            SelectionType::Single(val) => {
                                let idx = coord_info.nearest_index(*val);
                                format!("[{}]", idx)
                            }
                            SelectionType::Range(min_val, max_val) => {
                                let (min_idx, max_idx) = coord_info.range_indices(*min_val, *max_val);
                                format!("[{}:{}]", min_idx, max_idx)
                            }
                            SelectionType::Index(idx) => {
                                format!("[{}]", idx)
                            }
                            SelectionType::IndexRange(min_idx, max_idx) => {
                                format!("[{}:{}]", min_idx, max_idx)
                            }
                        };
                        parts.push(range_str);
                    }
                }
            }
            
            if !parts.is_empty() {
                let constraint = format!("{}{}", variable, parts.join(""));
                return self.dataset.get_variable(variable, Some(constraint)).await;
            }
        } else {
            // For regular variables, use SimpleConstraintBuilder
            let mut builder = SimpleConstraintBuilder::new();
            
            for (coord_name, selection) in &coord_selections {
                if let Some(coord_info) = self.coordinates.get(coord_name) {
                    match selection {
                        SelectionType::Single(val) => {
                            let idx = coord_info.nearest_index(*val);
                            builder = builder.add_single(coord_name, idx);
                        }
                        SelectionType::Range(min_val, max_val) => {
                            let (min_idx, max_idx) = coord_info.range_indices(*min_val, *max_val);
                            builder = builder.add_range(coord_name, min_idx, max_idx);
                        }
                        SelectionType::Index(idx) => {
                            builder = builder.add_single(coord_name, *idx);
                        }
                        SelectionType::IndexRange(min_idx, max_idx) => {
                            builder = builder.add_range(coord_name, *min_idx, *max_idx);
                        }
                    }
                }
            }
            
            let constraint = builder.build();
            if !constraint.is_empty() {
                return self.dataset.get_variable(variable, Some(constraint)).await;
            }
        }
        
        // Fallback to no constraints
        self.dataset.get_variable(variable, None).await
    }
    
    /// Select data using index ranges (for advanced users)
    #[wasm_bindgen(js_name = isel)]
    pub async fn isel(&self, variable: &str, selections: &Object) -> Result<Object, JsValue> {
        let mut coord_selections = HashMap::new();
        
        for key in js_sys::Object::keys(selections) {
            let key_str = key.as_string().unwrap();
            let value = Reflect::get(selections, &key)?;
            
            if value.is_object() {
                // Range selection: {min: 0, max: 10}
                let min_val = Reflect::get(&value, &JsValue::from_str("min"))?;
                let max_val = Reflect::get(&value, &JsValue::from_str("max"))?;
                
                if min_val.is_truthy() && max_val.is_truthy() && min_val.as_f64().is_some() && max_val.as_f64().is_some() {
                    let min = min_val.as_f64().unwrap() as usize;
                    let max = max_val.as_f64().unwrap() as usize;
                    coord_selections.insert(key_str, SelectionType::IndexRange(min, max));
                }
            } else if value.is_truthy() && value.as_f64().is_some() {
                // Single index selection
                let idx = value.as_f64().unwrap() as usize;
                coord_selections.insert(key_str, SelectionType::Index(idx));
            }
        }
        
        // Use the same selection logic as sel() but with index types
        self.sel_internal(variable, coord_selections).await
    }
    
    /// Get information about a specific coordinate
    #[wasm_bindgen(js_name = getCoordinate)]
    pub fn get_coordinate(&self, name: &str) -> Result<String, JsValue> {
        let coord_info = self.coordinates.get(name)
            .ok_or_else(|| JsValue::from_str(&format!("Coordinate '{}' not found", name)))?;
        
        serde_json::to_string(coord_info)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
}

impl XArrayDataset {
    /// Internal selection method
    async fn sel_internal(&self, variable: &str, coord_selections: HashMap<String, SelectionType>) -> Result<Object, JsValue> {
        let is_grid_variable = self.grid_variables.contains(variable);
        
        if is_grid_variable {
            // Grid variable constraint building
            let mut parts: Vec<String> = Vec::new();
            let standard_dims = ["longitude", "latitude", "time", "step"];
            
            for &dim_name in &standard_dims {
                if let Some(selection) = coord_selections.get(dim_name) {
                    let range_str = match selection {
                        SelectionType::Single(val) => {
                            if let Some(coord_info) = self.coordinates.get(dim_name) {
                                let idx = coord_info.nearest_index(*val);
                                format!("[{}]", idx)
                            } else {
                                continue;
                            }
                        }
                        SelectionType::Range(min_val, max_val) => {
                            if let Some(coord_info) = self.coordinates.get(dim_name) {
                                let (min_idx, max_idx) = coord_info.range_indices(*min_val, *max_val);
                                format!("[{}:{}]", min_idx, max_idx)
                            } else {
                                continue;
                            }
                        }
                        SelectionType::Index(idx) => {
                            format!("[{}]", idx)
                        }
                        SelectionType::IndexRange(min_idx, max_idx) => {
                            format!("[{}:{}]", min_idx, max_idx)
                        }
                    };
                    parts.push(range_str);
                }
            }
            
            if !parts.is_empty() {
                let constraint = format!("{}{}", variable, parts.join(""));
                return self.dataset.get_variable(variable, Some(constraint)).await;
            }
        } else {
            // Regular variable constraint building
            let mut builder = SimpleConstraintBuilder::new();
            
            for (coord_name, selection) in &coord_selections {
                match selection {
                    SelectionType::Single(val) => {
                        if let Some(coord_info) = self.coordinates.get(coord_name) {
                            let idx = coord_info.nearest_index(*val);
                            builder = builder.add_single(coord_name, idx);
                        }
                    }
                    SelectionType::Range(min_val, max_val) => {
                        if let Some(coord_info) = self.coordinates.get(coord_name) {
                            let (min_idx, max_idx) = coord_info.range_indices(*min_val, *max_val);
                            builder = builder.add_range(coord_name, min_idx, max_idx);
                        }
                    }
                    SelectionType::Index(idx) => {
                        builder = builder.add_single(coord_name, *idx);
                    }
                    SelectionType::IndexRange(min_idx, max_idx) => {
                        builder = builder.add_range(coord_name, *min_idx, *max_idx);
                    }
                }
            }
            
            let constraint = builder.build();
            if !constraint.is_empty() {
                return self.dataset.get_variable(variable, Some(constraint)).await;
            }
        }
        
        // Fallback
        self.dataset.get_variable(variable, None).await
    }
    
    /// Parse DDS content to identify coordinates and grid variables
    fn parse_dds_structure(dds_content: &str) -> Result<(Vec<String>, std::collections::HashSet<String>), JsValue> {
        let mut coordinate_vars: Vec<String> = Vec::new();
        let mut grid_vars = std::collections::HashSet::new();
        
        // Find coordinate variables (1D arrays with same name as dimension)
        for line in dds_content.lines() {
            let trimmed = line.trim();
            
            // Match patterns like: Float64 longitude[longitude = 1440];
            if trimmed.contains("[") && trimmed.contains("=") && trimmed.ends_with(";") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let var_name = parts[1].split('[').next().unwrap_or("");
                    if let Some(bracket_content) = trimmed.split('[').nth(1) {
                        if let Some(dim_part) = bracket_content.split('=').next() {
                            let dim_name = dim_part.trim();
                            if var_name == dim_name && !var_name.is_empty() {
                                coordinate_vars.push(var_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // Find Grid variables
        let lines: Vec<&str> = dds_content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("Grid") && line.contains("{") {
                // Find the closing brace and extract grid name
                let mut brace_depth = 0;
                for j in i..lines.len() {
                    let current_line = lines[j].trim();
                    brace_depth += current_line.chars().filter(|&c| c == '{').count();
                    brace_depth -= current_line.chars().filter(|&c| c == '}').count();
                    
                    if brace_depth == 0 && current_line.contains("}") && current_line.contains(";") {
                        if let Some(semi_pos) = current_line.find(';') {
                            let name_part = current_line[..semi_pos].trim();
                            if let Some(space_pos) = name_part.rfind(' ') {
                                let grid_name = name_part[space_pos + 1..].trim();
                                if !grid_name.is_empty() {
                                    grid_vars.insert(grid_name.to_string());
                                }
                            } else if let Some(brace_pos) = name_part.find('}') {
                                let grid_name = name_part[brace_pos + 1..].trim();
                                if !grid_name.is_empty() {
                                    grid_vars.insert(grid_name.to_string());
                                }
                            }
                        }
                        i = j;
                        break;
                    }
                }
            }
            i += 1;
        }
        
        Ok((coordinate_vars, grid_vars))
    }
    
    /// Load coordinate data for a single coordinate variable  
    async fn load_coordinate(dataset: &ImmutableDataset, coord_name: &str) -> Result<CoordinateInfo, JsValue> {
        // Load coordinate data - use Promise.race for timeout in JavaScript
        let coord_data = dataset.get_variable(coord_name, None).await?;
        
        // Extract data from the returned object
        let data_array = js_sys::Reflect::get(&coord_data, &JsValue::from_str("data"))?;
        let data_length = js_sys::Reflect::get(&data_array, &JsValue::from_str("length"))?
            .as_f64().unwrap_or(0.0) as usize;
        
        let mut values = Vec::with_capacity(data_length);
        for i in 0..data_length {
            let val = js_sys::Reflect::get(&data_array, &JsValue::from_f64(i as f64))?
                .as_f64().unwrap_or(0.0);
            values.push(val);
        }
        
        Ok(CoordinateInfo {
            name: coord_name.to_string(),
            size: values.len(),
            values,
            units: None, // Could be extracted from attributes
            long_name: None, // Could be extracted from attributes
        })
    }
}

/// Helper function to create an XArray-style dataset
#[wasm_bindgen(js_name = createXArrayDataset)]
pub async fn create_xarray_dataset(base_url: &str) -> Result<XArrayDataset, JsValue> {
    XArrayDataset::from_url(base_url).await
}