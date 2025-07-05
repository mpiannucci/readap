mod url_builder;
mod utils;

use js_sys::{
    Array, Float32Array, Float64Array, Int16Array, Int32Array, Int8Array, Uint16Array, Uint32Array,
};
use readap::data::{DataType, DataValue};
use readap::{parse_das_attributes, DdsDataset, DodsDataset};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub use url_builder::UrlBuilder;

// TypeScript-friendly data structures using serde
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub name: String,
    pub size: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsValueInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<Vec<Dimension>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<DdsValueInfo>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsDatasetInfo {
    pub name: String,
    pub values: Vec<DdsValueInfo>,
    pub variables: Vec<String>,
    pub coordinates: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DasAttributeInfo {
    pub name: String,
    pub data_type: String,
    pub value: String,
}

#[wasm_bindgen]
pub fn parse_dds(content: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();

    match DdsDataset::from_bytes(content) {
        Ok(dataset) => {
            let mut values = Vec::new();

            // Convert DDS values to our serializable structs
            for value in &dataset.values {
                let dds_value = convert_dds_value_to_info(value)?;
                values.push(dds_value);
            }

            let result = DdsDatasetInfo {
                name: dataset.name.clone(),
                values,
                variables: dataset.list_variables(),
                coordinates: dataset.list_coordinates(),
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {e}"))),
    }
}

#[wasm_bindgen]
pub fn parse_das(content: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();

    match parse_das_attributes(content) {
        Ok(attributes) => {
            // Use a plain JavaScript object instead of HashMap
            let js_obj = js_sys::Object::new();

            for (var_name, var_attrs) in attributes {
                let var_obj = js_sys::Object::new();
                for (attr_name, attr) in var_attrs {
                    let attr_info = DasAttributeInfo {
                        name: attr.name.clone(),
                        data_type: data_type_to_string(&attr.data_type),
                        value: data_value_to_string(&attr.value),
                    };
                    let attr_js = serde_wasm_bindgen::to_value(&attr_info).map_err(|e| {
                        JsValue::from_str(&format!("Attribute serialization error: {e}"))
                    })?;
                    js_sys::Reflect::set(&var_obj, &JsValue::from_str(&attr_name), &attr_js)
                        .map_err(|e| {
                            JsValue::from_str(&format!("Failed to set attribute: {e:?}"))
                        })?;
                }
                js_sys::Reflect::set(&js_obj, &JsValue::from_str(&var_name), &var_obj.into())
                    .map_err(|e| JsValue::from_str(&format!("Failed to set variable: {e:?}")))?;
            }

            Ok(js_obj.into())
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {e}"))),
    }
}

#[wasm_bindgen]
pub struct DodsData {
    dataset: DodsDataset<'static>,
    // Keep the bytes alive for zero-copy access
    _bytes: Vec<u8>,
}

#[wasm_bindgen]
impl DodsData {
    #[wasm_bindgen(js_name = getVariables)]
    pub fn get_variables(&self) -> Array {
        let variables = Array::new();
        for var_name in self.dataset.variables() {
            variables.push(&JsValue::from_str(&var_name));
        }
        variables
    }

    #[wasm_bindgen(js_name = getVariableData)]
    pub fn get_variable_data(&self, name: &str) -> Result<JsValue, JsValue> {
        let data_array = self
            .dataset
            .variable_data(name)
            .map_err(|e| JsValue::from_str(&format!("Failed to get variable data: {e}")))?;

        create_typed_array_from_data_array(&data_array)
    }

    #[wasm_bindgen(js_name = getVariableInfo)]
    pub fn get_variable_info(&self, name: &str) -> Result<JsValue, JsValue> {
        let index = self
            .dataset
            .variable_index(name)
            .ok_or_else(|| JsValue::from_str("Variable not found"))?;

        let dds_value = &self.dataset.dds.values[index];
        let value_info = convert_dds_value_to_info(dds_value)?;
        serde_wasm_bindgen::to_value(&value_info)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    #[wasm_bindgen(js_name = getDatasetInfo)]
    pub fn get_dataset_info(&self) -> Result<JsValue, JsValue> {
        let mut values = Vec::new();
        for value in &self.dataset.dds.values {
            let value_info = convert_dds_value_to_info(value)?;
            values.push(value_info);
        }

        let dataset_info = DdsDatasetInfo {
            name: self.dataset.dds.name.clone(),
            values,
            variables: self.dataset.dds.list_variables(),
            coordinates: self.dataset.dds.list_coordinates(),
        };

        serde_wasm_bindgen::to_value(&dataset_info)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }
}

#[wasm_bindgen]
pub fn parse_dods(bytes: &[u8]) -> Result<DodsData, JsValue> {
    utils::set_panic_hook();

    // Copy bytes to owned Vec to ensure lifetime
    let owned_bytes = bytes.to_vec();

    // SAFETY: We're extending the lifetime here, but we keep the owned_bytes
    // alive in the DodsData struct, so this is safe
    let bytes_ref: &'static [u8] =
        unsafe { std::slice::from_raw_parts(owned_bytes.as_ptr(), owned_bytes.len()) };

    match DodsDataset::from_bytes(bytes_ref) {
        Ok(dataset) => Ok(DodsData {
            dataset,
            _bytes: owned_bytes,
        }),
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {e}"))),
    }
}

fn create_typed_array_from_data_array(
    data_array: &readap::data::DataArray,
) -> Result<JsValue, JsValue> {
    match data_array {
        readap::data::DataArray::Byte(vec) => {
            let array = Int8Array::new_with_length(vec.len() as u32);
            for (i, &val) in vec.iter().enumerate() {
                array.set_index(i as u32, val);
            }
            Ok(array.into())
        }
        readap::data::DataArray::Int16(vec) => {
            let array = Int16Array::new_with_length(vec.len() as u32);
            for (i, &val) in vec.iter().enumerate() {
                array.set_index(i as u32, val);
            }
            Ok(array.into())
        }
        readap::data::DataArray::UInt16(vec) => {
            let array = Uint16Array::new_with_length(vec.len() as u32);
            for (i, &val) in vec.iter().enumerate() {
                array.set_index(i as u32, val);
            }
            Ok(array.into())
        }
        readap::data::DataArray::Int32(vec) => {
            let array = Int32Array::new_with_length(vec.len() as u32);
            for (i, &val) in vec.iter().enumerate() {
                array.set_index(i as u32, val);
            }
            Ok(array.into())
        }
        readap::data::DataArray::UInt32(vec) => {
            let array = Uint32Array::new_with_length(vec.len() as u32);
            for (i, &val) in vec.iter().enumerate() {
                array.set_index(i as u32, val);
            }
            Ok(array.into())
        }
        readap::data::DataArray::Float32(vec) => {
            let array = Float32Array::new_with_length(vec.len() as u32);
            for (i, &val) in vec.iter().enumerate() {
                array.set_index(i as u32, val);
            }
            Ok(array.into())
        }
        readap::data::DataArray::Float64(vec) => {
            let array = Float64Array::new_with_length(vec.len() as u32);
            for (i, &val) in vec.iter().enumerate() {
                array.set_index(i as u32, val);
            }
            Ok(array.into())
        }
        readap::data::DataArray::String(vec) => {
            let array = Array::new();
            for s in vec {
                array.push(&JsValue::from_str(s));
            }
            Ok(array.into())
        }
        readap::data::DataArray::URL(vec) => {
            let array = Array::new();
            for s in vec {
                array.push(&JsValue::from_str(s));
            }
            Ok(array.into())
        }
    }
}

// Helper functions
fn convert_dds_value_to_info(value: &readap::DdsValue) -> Result<DdsValueInfo, JsValue> {
    let name = value.name();
    let value_type = match value {
        readap::DdsValue::Array(_) => "Array",
        readap::DdsValue::Grid(_) => "Grid",
        readap::DdsValue::Structure(_) => "Structure",
        readap::DdsValue::Sequence(_) => "Sequence",
    }
    .to_string();

    let data_type = match value {
        readap::DdsValue::Array(array) => Some(data_type_to_string(&array.data_type)),
        readap::DdsValue::Grid(grid) => Some(data_type_to_string(&grid.array.data_type)),
        _ => None,
    };

    // Add coordinates if available
    let coordinates = match value {
        readap::DdsValue::Array(array) => {
            if !array.coords.is_empty() {
                Some(
                    array
                        .coords
                        .iter()
                        .map(|(name, size)| Dimension {
                            name: name.clone(),
                            size: *size,
                        })
                        .collect(),
                )
            } else {
                None
            }
        }
        readap::DdsValue::Grid(grid) => {
            if !grid.coords.is_empty() {
                Some(
                    grid.coords
                        .iter()
                        .map(|coord| Dimension {
                            name: coord.name.clone(),
                            size: coord.array_length(),
                        })
                        .collect(),
                )
            } else {
                None
            }
        }
        readap::DdsValue::Structure(structure) => {
            // For structures, we need to handle nested fields
            let fields: Result<Vec<DdsValueInfo>, JsValue> = structure
                .fields
                .iter()
                .map(convert_dds_value_to_info)
                .collect();
            return Ok(DdsValueInfo {
                name,
                value_type,
                data_type,
                coordinates: None,
                fields: Some(fields?),
            });
        }
        readap::DdsValue::Sequence(sequence) => {
            // For sequences, we need to handle nested fields
            let fields: Result<Vec<DdsValueInfo>, JsValue> = sequence
                .fields
                .iter()
                .map(convert_dds_value_to_info)
                .collect();
            return Ok(DdsValueInfo {
                name,
                value_type,
                data_type,
                coordinates: None,
                fields: Some(fields?),
            });
        }
    };

    Ok(DdsValueInfo {
        name,
        value_type,
        data_type,
        coordinates,
        fields: None,
    })
}

fn data_type_to_string(data_type: &DataType) -> String {
    match data_type {
        DataType::Byte => "Byte".to_string(),
        DataType::Int16 => "Int16".to_string(),
        DataType::UInt16 => "UInt16".to_string(),
        DataType::Int32 => "Int32".to_string(),
        DataType::UInt32 => "UInt32".to_string(),
        DataType::Float32 => "Float32".to_string(),
        DataType::Float64 => "Float64".to_string(),
        DataType::String => "String".to_string(),
        DataType::URL => "URL".to_string(),
    }
}

fn data_value_to_string(value: &DataValue) -> String {
    match value {
        DataValue::Byte(v) => v.to_string(),
        DataValue::Int16(v) => v.to_string(),
        DataValue::UInt16(v) => v.to_string(),
        DataValue::Int32(v) => v.to_string(),
        DataValue::UInt32(v) => v.to_string(),
        DataValue::Float32(v) => v.to_string(),
        DataValue::Float64(v) => v.to_string(),
        DataValue::String(v) => v.clone(),
        DataValue::URL(v) => v.clone(),
    }
}
