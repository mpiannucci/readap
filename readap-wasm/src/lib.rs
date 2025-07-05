mod url_builder;
mod utils;

use js_sys::{
    Array, Float32Array, Float64Array, Int16Array, Int32Array, Int8Array, Object, Reflect,
    Uint16Array, Uint32Array,
};
use readap::data::{DataType, DataValue};
use readap::{parse_das_attributes, DasAttribute, DdsDataset, DodsDataset};
use wasm_bindgen::prelude::*;

pub use url_builder::UrlBuilder;

#[wasm_bindgen]
pub fn parse_dds(content: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();

    match DdsDataset::from_bytes(content) {
        Ok(dataset) => {
            let obj = Object::new();

            // Set dataset name
            Reflect::set(&obj, &"name".into(), &JsValue::from_str(&dataset.name))
                .map_err(|_| JsValue::from_str("Failed to set name"))?;

            // Convert variables
            let vars_array = Array::new();
            for value in &dataset.values {
                let var_obj = dds_value_to_js(value)?;
                vars_array.push(&var_obj);
            }
            Reflect::set(&obj, &"values".into(), &vars_array)
                .map_err(|_| JsValue::from_str("Failed to set values"))?;

            // Add utility methods as properties
            let variables = Array::new();
            for var_name in dataset.list_variables() {
                variables.push(&JsValue::from_str(&var_name));
            }
            Reflect::set(&obj, &"variables".into(), &variables)
                .map_err(|_| JsValue::from_str("Failed to set variables list"))?;

            let coordinates = Array::new();
            for coord_name in dataset.list_coordinates() {
                coordinates.push(&JsValue::from_str(&coord_name));
            }
            Reflect::set(&obj, &"coordinates".into(), &coordinates)
                .map_err(|_| JsValue::from_str("Failed to set coordinates list"))?;

            Ok(obj.into())
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {e}"))),
    }
}

#[wasm_bindgen]
pub fn parse_das(content: &str) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();

    match parse_das_attributes(content) {
        Ok(attributes) => {
            let obj = Object::new();

            for (var_name, var_attrs) in attributes {
                let var_obj = Object::new();
                for (attr_name, attr) in var_attrs {
                    let attr_obj = das_attribute_to_js(&attr)?;
                    Reflect::set(&var_obj, &JsValue::from_str(&attr_name), &attr_obj)
                        .map_err(|_| JsValue::from_str("Failed to set attribute"))?;
                }
                Reflect::set(&obj, &JsValue::from_str(&var_name), &var_obj)
                    .map_err(|_| JsValue::from_str("Failed to set variable"))?;
            }

            Ok(obj.into())
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
        dds_value_to_js(dds_value)
    }

    #[wasm_bindgen(js_name = getDatasetInfo)]
    pub fn get_dataset_info(&self) -> Result<JsValue, JsValue> {
        let obj = Object::new();

        Reflect::set(
            &obj,
            &"name".into(),
            &JsValue::from_str(&self.dataset.dds.name),
        )
        .map_err(|_| JsValue::from_str("Failed to set name"))?;

        let vars_array = Array::new();
        for value in &self.dataset.dds.values {
            let var_obj = dds_value_to_js(value)?;
            vars_array.push(&var_obj);
        }
        Reflect::set(&obj, &"values".into(), &vars_array)
            .map_err(|_| JsValue::from_str("Failed to set values"))?;

        let variables = Array::new();
        for var_name in self.dataset.dds.list_variables() {
            variables.push(&JsValue::from_str(&var_name));
        }
        Reflect::set(&obj, &"variables".into(), &variables)
            .map_err(|_| JsValue::from_str("Failed to set variables list"))?;

        let coordinates = Array::new();
        for coord_name in self.dataset.dds.list_coordinates() {
            coordinates.push(&JsValue::from_str(&coord_name));
        }
        Reflect::set(&obj, &"coordinates".into(), &coordinates)
            .map_err(|_| JsValue::from_str("Failed to set coordinates list"))?;

        Ok(obj.into())
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
fn dds_value_to_js(value: &readap::DdsValue) -> Result<JsValue, JsValue> {
    let obj = Object::new();

    Reflect::set(&obj, &"name".into(), &JsValue::from_str(&value.name()))
        .map_err(|_| JsValue::from_str("Failed to set value name"))?;

    match value {
        readap::DdsValue::Array(array) => {
            Reflect::set(&obj, &"type".into(), &JsValue::from_str("Array"))
                .map_err(|_| JsValue::from_str("Failed to set type"))?;

            Reflect::set(
                &obj,
                &"dataType".into(),
                &JsValue::from_str(&data_type_to_string(&array.data_type)),
            )
            .map_err(|_| JsValue::from_str("Failed to set data type"))?;

            let coords_array = Array::new();
            for (coord_name, coord_size) in &array.coords {
                let coord_obj = Object::new();
                Reflect::set(&coord_obj, &"name".into(), &JsValue::from_str(coord_name))
                    .map_err(|_| JsValue::from_str("Failed to set coord name"))?;
                Reflect::set(
                    &coord_obj,
                    &"size".into(),
                    &JsValue::from_f64(*coord_size as f64),
                )
                .map_err(|_| JsValue::from_str("Failed to set coord size"))?;
                coords_array.push(&coord_obj);
            }
            Reflect::set(&obj, &"coordinates".into(), &coords_array)
                .map_err(|_| JsValue::from_str("Failed to set coordinates"))?;
        }
        readap::DdsValue::Grid(grid) => {
            Reflect::set(&obj, &"type".into(), &JsValue::from_str("Grid"))
                .map_err(|_| JsValue::from_str("Failed to set type"))?;

            Reflect::set(
                &obj,
                &"dataType".into(),
                &JsValue::from_str(&data_type_to_string(&grid.array.data_type)),
            )
            .map_err(|_| JsValue::from_str("Failed to set data type"))?;

            let coords_array = Array::new();
            for coord in &grid.coords {
                let coord_obj = Object::new();
                Reflect::set(&coord_obj, &"name".into(), &JsValue::from_str(&coord.name))
                    .map_err(|_| JsValue::from_str("Failed to set coord name"))?;
                Reflect::set(
                    &coord_obj,
                    &"size".into(),
                    &JsValue::from_f64(coord.array_length() as f64),
                )
                .map_err(|_| JsValue::from_str("Failed to set coord size"))?;
                coords_array.push(&coord_obj);
            }
            Reflect::set(&obj, &"coordinates".into(), &coords_array)
                .map_err(|_| JsValue::from_str("Failed to set coordinates"))?;
        }
        readap::DdsValue::Structure(structure) => {
            Reflect::set(&obj, &"type".into(), &JsValue::from_str("Structure"))
                .map_err(|_| JsValue::from_str("Failed to set type"))?;

            let fields_array = Array::new();
            for field in &structure.fields {
                let field_obj = dds_value_to_js(field)?;
                fields_array.push(&field_obj);
            }
            Reflect::set(&obj, &"fields".into(), &fields_array)
                .map_err(|_| JsValue::from_str("Failed to set fields"))?;
        }
        readap::DdsValue::Sequence(sequence) => {
            Reflect::set(&obj, &"type".into(), &JsValue::from_str("Sequence"))
                .map_err(|_| JsValue::from_str("Failed to set type"))?;

            let fields_array = Array::new();
            for field in &sequence.fields {
                let field_obj = dds_value_to_js(field)?;
                fields_array.push(&field_obj);
            }
            Reflect::set(&obj, &"fields".into(), &fields_array)
                .map_err(|_| JsValue::from_str("Failed to set fields"))?;
        }
    }

    Ok(obj.into())
}

fn das_attribute_to_js(attr: &DasAttribute) -> Result<JsValue, JsValue> {
    let obj = Object::new();

    Reflect::set(&obj, &"name".into(), &JsValue::from_str(&attr.name))
        .map_err(|_| JsValue::from_str("Failed to set attribute name"))?;

    Reflect::set(
        &obj,
        &"dataType".into(),
        &JsValue::from_str(&data_type_to_string(&attr.data_type)),
    )
    .map_err(|_| JsValue::from_str("Failed to set attribute data type"))?;

    // Set the actual value
    let value = data_value_to_js(&attr.value)?;
    Reflect::set(&obj, &"value".into(), &value)
        .map_err(|_| JsValue::from_str("Failed to set attribute value"))?;

    Ok(obj.into())
}

fn data_value_to_js(value: &DataValue) -> Result<JsValue, JsValue> {
    let js_value = match value {
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
    Ok(js_value)
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
