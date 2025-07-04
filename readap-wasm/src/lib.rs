mod utils;
mod url_builder;

use wasm_bindgen::prelude::*;
use readap::{DdsDataset, DasAttribute, DdsValue, parse_das_attributes, DodsDataset, query::{VariableInfo, CoordinateInfo}, data::{DataType, DataValue}};
use js_sys::{Array, Object, Reflect};

pub use url_builder::JsUrlBuilder;

#[wasm_bindgen]
pub fn parse_dds(content: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    
    match DdsDataset::from_bytes(content) {
        Ok(dataset) => {
            let obj = Object::new();
            
            // Set dataset name
            Reflect::set(&obj, &"name".into(), &JsValue::from_str(&dataset.name))
                .map_err(|_| JsValue::from_str("Failed to set name"))?;
            
            // Convert values (DdsValue instances)
            let values_array = Array::new();
            for value in &dataset.values {
                let value_obj = dds_value_to_js(value)?;
                values_array.push(&value_obj);
            }
            Reflect::set(&obj, &"values".into(), &values_array)
                .map_err(|_| JsValue::from_str("Failed to set values"))?;
            
            // Add helper methods results
            let variables = dataset.list_variables();
            let vars_array = Array::new();
            for var in variables {
                vars_array.push(&JsValue::from_str(&var));
            }
            Reflect::set(&obj, &"variables".into(), &vars_array)
                .map_err(|_| JsValue::from_str("Failed to set variables list"))?;
            
            let coordinates = dataset.list_coordinates();
            let coords_array = Array::new();
            for coord in coordinates {
                coords_array.push(&JsValue::from_str(&coord));
            }
            Reflect::set(&obj, &"coordinates".into(), &coords_array)
                .map_err(|_| JsValue::from_str("Failed to set coordinates list"))?;
            
            Ok(obj.into())
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {:?}", e)))
    }
}

#[wasm_bindgen]
pub fn parse_das(content: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    
    match parse_das_attributes(content) {
        Ok(attributes) => {
            let obj = Object::new();
            
            // attributes is HashMap<String, HashMap<String, DasAttribute>>
            for (var_name, var_attrs) in attributes {
                let var_obj = Object::new();
                for (attr_name, attr) in var_attrs {
                    let attr_obj = das_attribute_to_js(&attr)?;
                    Reflect::set(&var_obj, &JsValue::from_str(&attr_name), &attr_obj)
                        .map_err(|_| JsValue::from_str("Failed to set attribute"))?;
                }
                Reflect::set(&obj, &JsValue::from_str(&var_name), &var_obj)
                    .map_err(|_| JsValue::from_str("Failed to set variable attributes"))?;
            }
            
            Ok(obj.into())
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {:?}", e)))
    }
}

#[wasm_bindgen]
pub fn parse_dods(bytes: &[u8]) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    
    match DodsDataset::from_bytes(bytes) {
        Ok(dods) => {
            let obj = Object::new();
            
            // Convert DDS dataset
            let dds_obj = Object::new();
            Reflect::set(&dds_obj, &"name".into(), &JsValue::from_str(&dods.dds.name))
                .map_err(|_| JsValue::from_str("Failed to set dataset name"))?;
            
            let values_array = Array::new();
            for value in &dods.dds.values {
                let value_obj = dds_value_to_js(value)?;
                values_array.push(&value_obj);
            }
            Reflect::set(&dds_obj, &"values".into(), &values_array)
                .map_err(|_| JsValue::from_str("Failed to set values"))?;
            
            Reflect::set(&obj, &"dds".into(), &dds_obj)
                .map_err(|_| JsValue::from_str("Failed to set dds"))?;
            
            // Add available variables list
            let vars = dods.variables();
            let vars_array = Array::new();
            for var in vars {
                vars_array.push(&JsValue::from_str(&var));
            }
            Reflect::set(&obj, &"variables".into(), &vars_array)
                .map_err(|_| JsValue::from_str("Failed to set variables"))?;
            
            Ok(obj.into())
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {:?}", e)))
    }
}

// Helper functions
fn dds_value_to_js(value: &DdsValue) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"name".into(), &JsValue::from_str(&value.name()))
        .map_err(|_| JsValue::from_str("Failed to set value name"))?;
    
    let (value_type, details) = match value {
        DdsValue::Array(array) => {
            let details = Object::new();
            Reflect::set(&details, &"dataType".into(), &JsValue::from_str(&data_type_to_str(&array.data_type)))
                .map_err(|_| JsValue::from_str("Failed to set data type"))?;
            
            let coords_array = Array::new();
            for (coord_name, coord_size) in &array.coords {
                let coord_obj = Object::new();
                Reflect::set(&coord_obj, &"name".into(), &JsValue::from_str(coord_name))
                    .map_err(|_| JsValue::from_str("Failed to set coord name"))?;
                Reflect::set(&coord_obj, &"size".into(), &JsValue::from_f64(*coord_size as f64))
                    .map_err(|_| JsValue::from_str("Failed to set coord size"))?;
                coords_array.push(&coord_obj);
            }
            Reflect::set(&details, &"coords".into(), &coords_array)
                .map_err(|_| JsValue::from_str("Failed to set coords"))?;
            
            ("Array", details)
        },
        DdsValue::Grid(grid) => {
            let details = Object::new();
            
            // Add array info
            let array_obj = Object::new();
            Reflect::set(&array_obj, &"name".into(), &JsValue::from_str(&grid.array.name))
                .map_err(|_| JsValue::from_str("Failed to set array name"))?;
            Reflect::set(&array_obj, &"dataType".into(), &JsValue::from_str(&data_type_to_str(&grid.array.data_type)))
                .map_err(|_| JsValue::from_str("Failed to set array data type"))?;
            Reflect::set(&details, &"array".into(), &array_obj)
                .map_err(|_| JsValue::from_str("Failed to set array"))?;
            
            // Add coordinate arrays
            let coords_array = Array::new();
            for coord in &grid.coords {
                let coord_obj = Object::new();
                Reflect::set(&coord_obj, &"name".into(), &JsValue::from_str(&coord.name))
                    .map_err(|_| JsValue::from_str("Failed to set coord name"))?;
                Reflect::set(&coord_obj, &"dataType".into(), &JsValue::from_str(&data_type_to_str(&coord.data_type)))
                    .map_err(|_| JsValue::from_str("Failed to set coord data type"))?;
                Reflect::set(&coord_obj, &"size".into(), &JsValue::from_f64(coord.array_length() as f64))
                    .map_err(|_| JsValue::from_str("Failed to set coord size"))?;
                coords_array.push(&coord_obj);
            }
            Reflect::set(&details, &"coords".into(), &coords_array)
                .map_err(|_| JsValue::from_str("Failed to set coords"))?;
            
            ("Grid", details)
        },
        DdsValue::Structure(structure) => {
            let details = Object::new();
            let fields_array = Array::new();
            for field in &structure.fields {
                let field_obj = dds_value_to_js(field)?;
                fields_array.push(&field_obj);
            }
            Reflect::set(&details, &"fields".into(), &fields_array)
                .map_err(|_| JsValue::from_str("Failed to set fields"))?;
            ("Structure", details)
        },
        DdsValue::Sequence(sequence) => {
            let details = Object::new();
            let fields_array = Array::new();
            for field in &sequence.fields {
                let field_obj = dds_value_to_js(field)?;
                fields_array.push(&field_obj);
            }
            Reflect::set(&details, &"fields".into(), &fields_array)
                .map_err(|_| JsValue::from_str("Failed to set fields"))?;
            ("Sequence", details)
        },
    };
    
    Reflect::set(&obj, &"type".into(), &JsValue::from_str(value_type))
        .map_err(|_| JsValue::from_str("Failed to set value type"))?;
    Reflect::set(&obj, &"details".into(), &details)
        .map_err(|_| JsValue::from_str("Failed to set details"))?;
    
    Ok(obj.into())
}

fn data_type_to_str(data_type: &DataType) -> &'static str {
    match data_type {
        DataType::Byte => "Byte",
        DataType::Int16 => "Int16",
        DataType::UInt16 => "UInt16",
        DataType::Int32 => "Int32",
        DataType::UInt32 => "UInt32",
        DataType::Float32 => "Float32",
        DataType::Float64 => "Float64",
        DataType::String => "String",
        DataType::URL => "URL",
    }
}

fn das_attribute_to_js(attr: &DasAttribute) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"name".into(), &JsValue::from_str(&attr.name))
        .map_err(|_| JsValue::from_str("Failed to set attribute name"))?;
    
    Reflect::set(&obj, &"type".into(), &JsValue::from_str(&data_type_to_str(&attr.data_type)))
        .map_err(|_| JsValue::from_str("Failed to set attribute type"))?;
    
    // Set the actual value
    let value = match &attr.value {
        DataValue::Byte(v) => JsValue::from_f64(*v as f64),
        DataValue::Int16(v) => JsValue::from_f64(*v as f64),
        DataValue::UInt16(v) => JsValue::from_f64(*v as f64),
        DataValue::Int32(v) => JsValue::from_f64(*v as f64),
        DataValue::UInt32(v) => JsValue::from_f64(*v as f64),
        DataValue::Float32(v) => JsValue::from_f64(*v as f64),
        DataValue::Float64(v) => JsValue::from_f64(*v),
        DataValue::String(v) => JsValue::from_str(v),
        DataValue::URL(v) => JsValue::from_str(v),
    };
    
    Reflect::set(&obj, &"value".into(), &value)
        .map_err(|_| JsValue::from_str("Failed to set attribute value"))?;
    
    Ok(obj.into())
}

#[wasm_bindgen]
pub fn create_query_builder(dds_content: &str, base_url: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    
    match DdsDataset::from_bytes(dds_content) {
        Ok(dataset) => {
            let obj = Object::new();
            
            // Store dataset info
            Reflect::set(&obj, &"baseUrl".into(), &JsValue::from_str(base_url))
                .map_err(|_| JsValue::from_str("Failed to set base URL"))?;
            
            // Add helper methods
            let variables = dataset.list_variables();
            let vars_array = Array::new();
            for var in variables {
                vars_array.push(&JsValue::from_str(&var));
            }
            Reflect::set(&obj, &"variables".into(), &vars_array)
                .map_err(|_| JsValue::from_str("Failed to set variables"))?;
            
            let coordinates = dataset.list_coordinates();
            let coords_array = Array::new();
            for coord in coordinates {
                coords_array.push(&JsValue::from_str(&coord));
            }
            Reflect::set(&obj, &"coordinates".into(), &coords_array)
                .map_err(|_| JsValue::from_str("Failed to set coordinates"))?;
            
            Ok(obj.into())
        }
        Err(e) => Err(JsValue::from_str(&format!("Failed to create query builder: {:?}", e)))
    }
}

#[wasm_bindgen]
pub fn get_variable_info(dds_content: &str, variable_name: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    
    match DdsDataset::from_bytes(dds_content) {
        Ok(dataset) => {
            if let Some(var_info) = dataset.get_variable_info(variable_name) {
                variable_info_to_js(&var_info)
            } else {
                Err(JsValue::from_str(&format!("Variable '{}' not found", variable_name)))
            }
        }
        Err(e) => Err(JsValue::from_str(&format!("Failed to parse DDS: {:?}", e)))
    }
}

#[wasm_bindgen]
pub fn get_coordinate_info(dds_content: &str, coordinate_name: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();
    
    match DdsDataset::from_bytes(dds_content) {
        Ok(dataset) => {
            if let Some(coord_info) = dataset.get_coordinate_info(coordinate_name) {
                coordinate_info_to_js(&coord_info)
            } else {
                Err(JsValue::from_str(&format!("Coordinate '{}' not found", coordinate_name)))
            }
        }
        Err(e) => Err(JsValue::from_str(&format!("Failed to parse DDS: {:?}", e)))
    }
}

fn variable_info_to_js(var_info: &VariableInfo) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"name".into(), &JsValue::from_str(&var_info.name))
        .map_err(|_| JsValue::from_str("Failed to set variable name"))?;
    
    Reflect::set(&obj, &"dataType".into(), &JsValue::from_str(&data_type_to_str(&var_info.data_type)))
        .map_err(|_| JsValue::from_str("Failed to set data type"))?;
    
    let var_type_str = match var_info.variable_type {
        readap::query::VariableType::Array => "Array",
        readap::query::VariableType::Grid => "Grid",
        readap::query::VariableType::Structure => "Structure",
        readap::query::VariableType::Sequence => "Sequence",
    };
    Reflect::set(&obj, &"variableType".into(), &JsValue::from_str(var_type_str))
        .map_err(|_| JsValue::from_str("Failed to set variable type"))?;
    
    let coords_array = Array::new();
    for coord in &var_info.coordinates {
        coords_array.push(&JsValue::from_str(coord));
    }
    Reflect::set(&obj, &"coordinates".into(), &coords_array)
        .map_err(|_| JsValue::from_str("Failed to set coordinates"))?;
    
    let dims_array = Array::new();
    for (dim_name, dim_size) in &var_info.dimensions {
        let dim_obj = Object::new();
        Reflect::set(&dim_obj, &"name".into(), &JsValue::from_str(dim_name))
            .map_err(|_| JsValue::from_str("Failed to set dimension name"))?;
        Reflect::set(&dim_obj, &"size".into(), &JsValue::from_f64(*dim_size as f64))
            .map_err(|_| JsValue::from_str("Failed to set dimension size"))?;
        dims_array.push(&dim_obj);
    }
    Reflect::set(&obj, &"dimensions".into(), &dims_array)
        .map_err(|_| JsValue::from_str("Failed to set dimensions"))?;
    
    Ok(obj.into())
}

fn coordinate_info_to_js(coord_info: &CoordinateInfo) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"name".into(), &JsValue::from_str(&coord_info.name))
        .map_err(|_| JsValue::from_str("Failed to set coordinate name"))?;
    
    Reflect::set(&obj, &"dataType".into(), &JsValue::from_str(&data_type_to_str(&coord_info.data_type)))
        .map_err(|_| JsValue::from_str("Failed to set data type"))?;
    
    Reflect::set(&obj, &"size".into(), &JsValue::from_f64(coord_info.size as f64))
        .map_err(|_| JsValue::from_str("Failed to set size"))?;
    
    let vars_array = Array::new();
    for var in &coord_info.variables_using {
        vars_array.push(&JsValue::from_str(var));
    }
    Reflect::set(&obj, &"variablesUsing".into(), &vars_array)
        .map_err(|_| JsValue::from_str("Failed to set variables using"))?;
    
    Ok(obj.into())
}