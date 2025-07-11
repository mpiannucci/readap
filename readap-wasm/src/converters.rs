use readap::{
    das::{DasAttribute, DasAttributes, DasVariable},
    data::{DataType, DataValue},
    dds::{DdsArray, DdsDataset, DdsGrid, DdsSequence, DdsStructure, DdsValue},
};
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

pub fn data_type_to_string(data_type: &DataType) -> String {
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

pub fn data_value_to_js_value(value: &DataValue) -> JsValue {
    match value {
        DataValue::Byte(v) => JsValue::from(*v),
        DataValue::Int16(v) => JsValue::from(*v),
        DataValue::UInt16(v) => JsValue::from(*v),
        DataValue::Int32(v) => JsValue::from(*v),
        DataValue::UInt32(v) => JsValue::from(*v),
        DataValue::Float32(v) => JsValue::from(*v),
        DataValue::Float64(v) => JsValue::from(*v),
        DataValue::String(v) => JsValue::from(v),
        DataValue::URL(v) => JsValue::from(v),
    }
}

pub fn das_attribute_to_js_object(attribute: &DasAttribute) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"dataType".into(), &data_type_to_string(&attribute.data_type).into())?;
    Reflect::set(&obj, &"name".into(), &attribute.name.clone().into())?;
    Reflect::set(&obj, &"value".into(), &data_value_to_js_value(&attribute.value))?;
    
    Ok(obj.into())
}

pub fn das_variable_to_js_object(variable: &DasVariable) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    for (name, attribute) in variable.iter() {
        let attr_obj = das_attribute_to_js_object(attribute)?;
        Reflect::set(&obj, &name.clone().into(), &attr_obj)?;
    }
    
    Ok(obj.into())
}

pub fn das_attributes_to_js_object(attributes: &DasAttributes) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    for (name, variable) in attributes.iter() {
        let var_obj = das_variable_to_js_object(variable)?;
        Reflect::set(&obj, &name.clone().into(), &var_obj)?;
    }
    
    Ok(obj.into())
}

// DDS converters
pub fn dds_array_to_js_object(array: &DdsArray) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"type".into(), &"Array".into())?;
    Reflect::set(&obj, &"name".into(), &array.name.clone().into())?;
    Reflect::set(&obj, &"dataType".into(), &data_type_to_string(&array.data_type).into())?;
    Reflect::set(&obj, &"arrayLength".into(), &array.array_length().into())?;
    Reflect::set(&obj, &"byteCount".into(), &array.byte_count().into())?;
    
    // Convert coordinates
    let coords_array = Array::new();
    for (coord_name, coord_size) in &array.coords {
        let coord_obj = Object::new();
        Reflect::set(&coord_obj, &"name".into(), &coord_name.clone().into())?;
        Reflect::set(&coord_obj, &"size".into(), &(*coord_size).into())?;
        coords_array.push(&coord_obj.into());
    }
    Reflect::set(&obj, &"coordinates".into(), &coords_array.into())?;
    
    Ok(obj.into())
}

pub fn dds_grid_to_js_object(grid: &DdsGrid) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"type".into(), &"Grid".into())?;
    Reflect::set(&obj, &"name".into(), &grid.name.clone().into())?;
    Reflect::set(&obj, &"byteCount".into(), &grid.byte_count().into())?;
    Reflect::set(&obj, &"coordsOffset".into(), &grid.coords_offset().into())?;
    
    // Convert main array
    let array_obj = dds_array_to_js_object(&grid.array)?;
    Reflect::set(&obj, &"array".into(), &array_obj)?;
    
    // Convert coordinate arrays
    let coords_array = Array::new();
    for coord in &grid.coords {
        let coord_obj = dds_array_to_js_object(coord)?;
        coords_array.push(&coord_obj);
    }
    Reflect::set(&obj, &"coordinates".into(), &coords_array.into())?;
    
    // Convert coordinate offsets
    let offsets_array = Array::new();
    for offset in grid.coord_offsets() {
        offsets_array.push(&offset.into());
    }
    Reflect::set(&obj, &"coordinateOffsets".into(), &offsets_array.into())?;
    
    Ok(obj.into())
}

pub fn dds_structure_to_js_object(structure: &DdsStructure) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"type".into(), &"Structure".into())?;
    Reflect::set(&obj, &"name".into(), &structure.name.clone().into())?;
    Reflect::set(&obj, &"byteCount".into(), &structure.byte_count().into())?;
    
    // Convert fields
    let fields_array = Array::new();
    for field in &structure.fields {
        let field_obj = dds_value_to_js_object(field)?;
        fields_array.push(&field_obj);
    }
    Reflect::set(&obj, &"fields".into(), &fields_array.into())?;
    
    Ok(obj.into())
}

pub fn dds_sequence_to_js_object(sequence: &DdsSequence) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"type".into(), &"Sequence".into())?;
    Reflect::set(&obj, &"name".into(), &sequence.name.clone().into())?;
    Reflect::set(&obj, &"byteCount".into(), &sequence.byte_count().into())?;
    
    // Convert fields
    let fields_array = Array::new();
    for field in &sequence.fields {
        let field_obj = dds_value_to_js_object(field)?;
        fields_array.push(&field_obj);
    }
    Reflect::set(&obj, &"fields".into(), &fields_array.into())?;
    
    Ok(obj.into())
}

pub fn dds_value_to_js_object(value: &DdsValue) -> Result<JsValue, JsValue> {
    match value {
        DdsValue::Array(array) => dds_array_to_js_object(array),
        DdsValue::Grid(grid) => dds_grid_to_js_object(grid),
        DdsValue::Structure(structure) => dds_structure_to_js_object(structure),
        DdsValue::Sequence(sequence) => dds_sequence_to_js_object(sequence),
    }
}

pub fn dds_dataset_to_js_object(dataset: &DdsDataset) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"name".into(), &dataset.name.clone().into())?;
    
    // Convert values
    let values_array = Array::new();
    for value in &dataset.values {
        let value_obj = dds_value_to_js_object(value)?;
        values_array.push(&value_obj);
    }
    Reflect::set(&obj, &"values".into(), &values_array.into())?;
    
    // Add metadata methods as properties
    let variables = Array::new();
    for var_name in dataset.list_variables() {
        variables.push(&var_name.into());
    }
    Reflect::set(&obj, &"variables".into(), &variables.into())?;
    
    let coordinates = Array::new();
    for coord_name in dataset.list_coordinates() {
        coordinates.push(&coord_name.into());
    }
    Reflect::set(&obj, &"coordinates".into(), &coordinates.into())?;
    
    Ok(obj.into())
}