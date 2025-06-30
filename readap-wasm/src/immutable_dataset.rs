/// Immutable Dataset API that avoids mutable self references
/// This design prevents "recursive use of an object detected" errors in Bun/Node.js
/// by always returning new instances instead of mutating existing ones

use crate::{ConstraintBuilder, CoordinateResolver, OpenDAPUrlBuilder, UniversalFetch, UniversalDodsParser};
use js_sys::{Array, Object, Reflect, Uint8Array};
// use readap::DdsDataset; // Not needed for simplified parsing
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

/// Dataset configuration that gets passed around instead of mutating
#[derive(Debug, Clone)]
struct DatasetConfig {
    base_url: String,
    das_data: Option<String>,
    dds_data: Option<String>,
    variables: HashMap<String, VariableInfo>,
    coordinate_cache: HashMap<String, Array>,
}

impl DatasetConfig {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            das_data: None,
            dds_data: None,
            variables: HashMap::new(),
            coordinate_cache: HashMap::new(),
        }
    }

    fn with_das_data(mut self, das_data: String) -> Self {
        self.das_data = Some(das_data);
        self
    }

    fn with_dds_data(mut self, dds_data: String) -> Self {
        self.dds_data = Some(dds_data);
        self
    }

    fn with_variables(mut self, variables: HashMap<String, VariableInfo>) -> Self {
        self.variables = variables;
        self
    }

    fn with_coordinates(mut self, var_name: String, coords: Array) -> Self {
        self.coordinate_cache.insert(var_name, coords);
        self
    }
}

/// Immutable OpenDAP Dataset - never mutates, always returns new instances
#[wasm_bindgen]
pub struct ImmutableDataset {
    config: DatasetConfig,
    base_url: String, // Store base_url separately to avoid cloning issues
    coordinate_resolver: CoordinateResolver,
    fetch_client: UniversalFetch,
    dods_parser: UniversalDodsParser,
}

/// Result of dataset operations that may create a new dataset instance
#[wasm_bindgen]
pub struct DatasetResult {
    dataset: Option<ImmutableDataset>,
    error: Option<String>,
}

#[wasm_bindgen]
impl DatasetResult {
    /// Check if the operation was successful
    #[wasm_bindgen(js_name = isSuccess)]
    pub fn is_success(&self) -> bool {
        self.dataset.is_some()
    }

    /// Get the resulting dataset (if successful)
    #[wasm_bindgen(js_name = getDataset)]
    pub fn get_dataset(self) -> Option<ImmutableDataset> {
        self.dataset
    }

    /// Get the error message (if failed)
    #[wasm_bindgen(js_name = getError)]
    pub fn get_error(&self) -> Option<String> {
        self.error.clone()
    }
}

#[wasm_bindgen]
impl ImmutableDataset {
    /// Create a new dataset from a base URL
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: &str) -> Result<ImmutableDataset, JsValue> {
        let config = DatasetConfig::new(base_url);
        let coordinate_resolver = CoordinateResolver::new();
        let fetch_client = UniversalFetch::new()?;
        let dods_parser = UniversalDodsParser::new();

        Ok(ImmutableDataset {
            config,
            base_url: base_url.to_string(),
            coordinate_resolver,
            fetch_client,
            dods_parser,
        })
    }

    /// Create a dataset and automatically load metadata
    #[wasm_bindgen(js_name = fromURL)]
    pub async fn from_url(base_url: &str) -> Result<ImmutableDataset, JsValue> {
        let mut dataset = Self::new(base_url)?;
        
        // Load DAS data
        let url_builder = OpenDAPUrlBuilder::new(base_url);
        let das_url = url_builder.das_url();
        let das_data = dataset.fetch_client.fetch_text(&das_url).await?;
        
        // Load DDS data
        let dds_url = url_builder.dds_url();
        let dds_data = dataset.fetch_client.fetch_text(&dds_url).await?;
        
        // Create new dataset with loaded metadata
        dataset = dataset.with_das_data(das_data)?;
        dataset = dataset.with_dds_data(dds_data)?;
        
        Ok(dataset)
    }

    /// Create a dataset from DAS data only
    #[wasm_bindgen(js_name = fromDAS)]
    pub fn from_das(das_data: &str) -> Result<ImmutableDataset, JsValue> {
        let mut dataset = Self::new("")?;
        dataset = dataset.with_das_data(das_data.to_string())?;
        Ok(dataset)
    }

    /// Create a dataset from DDS data only
    #[wasm_bindgen(js_name = fromDDS)]
    pub fn from_dds(dds_data: &str) -> Result<ImmutableDataset, JsValue> {
        let mut dataset = Self::new("")?;
        dataset = dataset.with_dds_data(dds_data.to_string())?;
        Ok(dataset)
    }

    /// Add DAS data and return new dataset instance
    #[wasm_bindgen(js_name = withDAS)]
    pub fn with_das_data(&self, das_data: String) -> Result<ImmutableDataset, JsValue> {
        let mut new_config = self.config.clone().with_das_data(das_data);
        
        // Parse DAS data to extract variables
        if let Some(das_content) = &new_config.das_data {
            let variables = Self::parse_das_variables(das_content)?;
            new_config = new_config.with_variables(variables);
        }

        Ok(ImmutableDataset {
            config: new_config,
            base_url: self.base_url.clone(),
            coordinate_resolver: CoordinateResolver::new(),
            fetch_client: UniversalFetch::new()?,
            dods_parser: UniversalDodsParser::new(),
        })
    }

    /// Add DDS data and return new dataset instance
    #[wasm_bindgen(js_name = withDDS)]
    pub fn with_dds_data(&self, dds_data: String) -> Result<ImmutableDataset, JsValue> {
        let mut new_config = self.config.clone().with_dds_data(dds_data);
        
        // Parse DDS data to extract variables
        if let Some(dds_content) = &new_config.dds_data {
            let variables = Self::parse_dds_variables(dds_content)?;
            new_config = new_config.with_variables(variables);
        }

        Ok(ImmutableDataset {
            config: new_config,
            base_url: self.base_url.clone(),
            coordinate_resolver: CoordinateResolver::new(),
            fetch_client: UniversalFetch::new()?,
            dods_parser: UniversalDodsParser::new(),
        })
    }

    /// Add coordinate data and return new dataset instance
    #[wasm_bindgen(js_name = withCoordinates)]
    pub fn with_coordinates(&self, var_name: &str, coords: &Array) -> Result<ImmutableDataset, JsValue> {
        let new_config = self.config.clone().with_coordinates(var_name.to_string(), coords.clone());

        Ok(ImmutableDataset {
            config: new_config,
            base_url: self.base_url.clone(),
            coordinate_resolver: CoordinateResolver::new(),
            fetch_client: UniversalFetch::new()?,
            dods_parser: UniversalDodsParser::new(),
        })
    }

    /// Parse DODS binary data using the universal parser
    #[wasm_bindgen(js_name = parseDODS)]
    pub fn parse_dods(&self, dods_data: &Uint8Array) -> Result<Object, JsValue> {
        self.dods_parser.parse_dods(dods_data)
    }

    /// Parse DODS binary data with detailed error information
    #[wasm_bindgen(js_name = parseDodsDetailed)]
    pub fn parse_dods_detailed(&self, dods_data: &Uint8Array) -> Object {
        self.dods_parser.parse_dods_detailed(dods_data)
    }

    /// Get variable names
    #[wasm_bindgen(js_name = getVariableNames)]
    pub fn get_variable_names(&self) -> Array {
        let names = Array::new();
        for name in self.config.variables.keys() {
            names.push(&JsValue::from_str(name));
        }
        names
    }

    /// Get variable information as JSON
    #[wasm_bindgen(js_name = getVariableInfo)]
    pub fn get_variable_info(&self, name: &str) -> Result<String, JsValue> {
        let info = self
            .config
            .variables
            .get(name)
            .ok_or_else(|| JsValue::from_str(&format!("Variable '{}' not found", name)))?;

        serde_json::to_string(info).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get all variables information as JSON
    #[wasm_bindgen(js_name = getVariablesInfo)]
    pub fn get_variables_info(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.config.variables).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create index-based selection constraint builder
    #[wasm_bindgen(js_name = isel)]
    pub fn isel(&self, selections: &Object) -> Result<ConstraintBuilder, JsValue> {
        let mut builder = ConstraintBuilder::new();
        builder = builder.isel(&selections.clone())?;
        Ok(builder)
    }

    /// Create value-based selection constraint builder  
    #[wasm_bindgen(js_name = sel)]
    pub fn sel(&self, selections: &Object) -> Result<ConstraintBuilder, JsValue> {
        let mut builder = ConstraintBuilder::new();
        builder = builder.sel(&selections.clone())?;
        
        // Resolve value-based constraints to index-based using coordinate resolver
        let resolved_builder = self.coordinate_resolver.resolve_constraints(&builder)?;
        Ok(resolved_builder)
    }

    /// Get variable data with automatic fetching
    #[wasm_bindgen(js_name = getVariable)]
    pub async fn get_variable(
        &self,
        var_name: &str,
        constraints: Option<String>,
    ) -> Result<Object, JsValue> {
        let url_builder = OpenDAPUrlBuilder::new(&self.base_url);
        let dods_url = if let Some(constraint_str) = constraints {
            url_builder.dods_url(Some(constraint_str))
        } else {
            url_builder.dods_url(None)
        };

        let binary_data = self.fetch_client.fetch_binary(&dods_url).await?;
        let uint8_data = Uint8Array::from(&binary_data[..]);
        
        let parsed_data = self.parse_dods(&uint8_data)?;
        
        // Extract the specific variable
        if let Ok(var_data) = Reflect::get(&parsed_data, &JsValue::from_str(var_name)) {
            Ok(var_data.into())
        } else {
            Err(JsValue::from_str(&format!("Variable '{}' not found in DODS response", var_name)))
        }
    }

    /// Get multiple variables with automatic fetching
    #[wasm_bindgen(js_name = getVariables)]
    pub async fn get_variables(
        &self,
        _var_names: &Array,
        constraints: Option<String>,
    ) -> Result<Object, JsValue> {
        let url_builder = OpenDAPUrlBuilder::new(&self.base_url);
        let dods_url = if let Some(constraint_str) = constraints {
            url_builder.dods_url(Some(constraint_str))
        } else {
            url_builder.dods_url(None)
        };

        let binary_data = self.fetch_client.fetch_binary(&dods_url).await?;
        let uint8_data = Uint8Array::from(&binary_data[..]);
        
        self.parse_dods(&uint8_data)
    }

    /// Get the base URL
    #[wasm_bindgen(js_name = baseUrl)]
    pub fn base_url(&self) -> String {
        self.config.base_url.clone()
    }

    /// Get DAS URL
    #[wasm_bindgen(js_name = dasUrl)]
    pub fn das_url(&self) -> String {
        let url_builder = OpenDAPUrlBuilder::new(&self.base_url);
        url_builder.das_url()
    }

    /// Get DDS URL
    #[wasm_bindgen(js_name = ddsUrl)]  
    pub fn dds_url(&self) -> String {
        let url_builder = OpenDAPUrlBuilder::new(&self.base_url);
        url_builder.dds_url()
    }

    /// Get DODS URL with optional constraints
    #[wasm_bindgen(js_name = dodsUrl)]
    pub fn dods_url(&self, constraints: Option<String>) -> String {
        let url_builder = OpenDAPUrlBuilder::new(&self.base_url);
        if let Some(constraint_str) = constraints {
            url_builder.dods_url(Some(constraint_str))
        } else {
            url_builder.dods_url(None)
        }
    }

    /// Get DODS URL with constraint builder
    #[wasm_bindgen(js_name = dodsUrlWithConstraints)]
    pub fn dods_url_with_constraints(&self, builder: &ConstraintBuilder) -> String {
        let url_builder = OpenDAPUrlBuilder::new(&self.base_url);
        let constraint_str = builder.build();
        url_builder.dods_url(Some(constraint_str))
    }
}

// Helper methods for parsing metadata
impl ImmutableDataset {
    fn parse_das_variables(das_data: &str) -> Result<HashMap<String, VariableInfo>, JsValue> {
        let mut variables = HashMap::new();
        
        // Simple DAS parsing - this is a simplified version
        // In a real implementation, you'd want more robust parsing
        for line in das_data.lines() {
            if line.trim().starts_with("//") || line.trim().is_empty() {
                continue;
            }
            
            // Look for attribute declarations
            if line.contains("{") && !line.contains("attributes") {
                let var_name = line.trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('{')
                    .to_string();
                    
                if !var_name.is_empty() && var_name != "attributes" {
                    let var_info = VariableInfo {
                        name: var_name.clone(),
                        data_type: "unknown".to_string(),
                        dimensions: Vec::new(),
                        attributes: HashMap::new(),
                    };
                    variables.insert(var_name, var_info);
                }
            }
        }
        
        Ok(variables)
    }

    fn parse_dds_variables(dds_data: &str) -> Result<HashMap<String, VariableInfo>, JsValue> {
        let mut variables = HashMap::new();
        
        // Simple DDS parsing for variable names
        for line in dds_data.lines() {
            let trimmed = line.trim();
            
            // Look for variable declarations like: Float32 temperature[time = 8][lat = 4][lon = 8];
            if (trimmed.contains("Float32") || trimmed.contains("Float64") || trimmed.contains("Int32")) && trimmed.contains("[") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let data_type = parts[0];
                    let var_part = parts[1];
                    
                    if let Some(bracket_pos) = var_part.find('[') {
                        let var_name = var_part[..bracket_pos].to_string();
                        
                        let var_info = VariableInfo {
                            name: var_name.clone(),
                            data_type: data_type.to_string(),
                            dimensions: Vec::new(), // Simplified - not parsing dimensions for now
                            attributes: HashMap::new(),
                        };
                        variables.insert(var_name, var_info);
                    }
                }
            }
        }

        Ok(variables)
    }
}

/// Helper function to create an immutable dataset
#[wasm_bindgen(js_name = createImmutableDataset)]
pub fn create_immutable_dataset(base_url: &str) -> Result<ImmutableDataset, JsValue> {
    ImmutableDataset::new(base_url)
}