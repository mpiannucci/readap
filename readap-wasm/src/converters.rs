use readap::{
    das::{DasAttribute, DasAttributes, DasVariable},
    data::{DataType, DataValue},
};
use js_sys::{Object, Reflect};
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