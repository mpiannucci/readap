mod utils;

use readap::{
    das::{parse_das_attributes, DasAttribute, DasAttributes, DasVariable},
    data::{DataType, DataValue},
    IndexRange as RustIndexRange, UrlBuilder as RustUrlBuilder,
};
use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct UrlBuilder {
    inner: RustUrlBuilder,
}

#[wasm_bindgen]
pub enum IndexRangeType {
    Single,
    Range,
}

#[wasm_bindgen]
pub struct IndexRange {
    range_type: IndexRangeType,
    value: Option<isize>,
    start: Option<isize>,
    end: Option<isize>,
    stride: Option<isize>,
}

#[wasm_bindgen]
impl IndexRange {
    #[wasm_bindgen(constructor)]
    pub fn from_single(value: isize) -> IndexRange {
        IndexRange {
            range_type: IndexRangeType::Single,
            value: Some(value),
            start: None,
            end: None,
            stride: None,
        }
    }

    #[wasm_bindgen(js_name = fromRange)]
    pub fn from_range(start: isize, end: isize, stride: Option<isize>) -> IndexRange {
        IndexRange {
            range_type: IndexRangeType::Range,
            value: None,
            start: Some(start),
            end: Some(end),
            stride,
        }
    }
}

impl Into<RustIndexRange> for IndexRange {
    fn into(self) -> RustIndexRange {
        match self.range_type {
            IndexRangeType::Single => RustIndexRange::Single(self.value.unwrap()),
            IndexRangeType::Range => RustIndexRange::Range {
                start: self.start.unwrap(),
                end: self.end.unwrap(),
                stride: self.stride,
            },
        }
    }
}

impl Into<RustIndexRange> for &IndexRange {
    fn into(self) -> RustIndexRange {
        match self.range_type {
            IndexRangeType::Single => RustIndexRange::Single(self.value.unwrap()),
            IndexRangeType::Range => RustIndexRange::Range {
                start: self.start.unwrap(),
                end: self.end.unwrap(),
                stride: self.stride,
            },
        }
    }
}

#[wasm_bindgen]
impl UrlBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: &str) -> UrlBuilder {
        UrlBuilder {
            inner: RustUrlBuilder::new(base_url),
        }
    }

    #[wasm_bindgen(js_name = dasUrl)]
    pub fn das_url(&self) -> String {
        self.inner.das_url()
    }

    #[wasm_bindgen(js_name = ddsUrl)]
    pub fn dds_url(&self) -> String {
        self.inner.dds_url()
    }

    #[wasm_bindgen(js_name = dodsUrl)]
    pub fn dods_url(&self) -> Result<String, String> {
        self.inner.dods_url().map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = addVariable)]
    pub fn add_variable(self, variable: &str) -> UrlBuilder {
        UrlBuilder {
            inner: self.inner.add_variable(variable),
        }
    }

    #[wasm_bindgen(js_name = addVariables)]
    pub fn add_variables(self, variables: Vec<String>) -> UrlBuilder {
        UrlBuilder {
            inner: self.inner.add_variables(variables),
        }
    }

    #[wasm_bindgen(js_name = addSingleIndex)]
    pub fn add_single_index(self, variable: &str, index: isize) -> UrlBuilder {
        UrlBuilder {
            inner: self.inner.add_single_index(variable, index),
        }
    }

    #[wasm_bindgen(js_name = addRange)]
    pub fn add_range(
        self,
        variable: &str,
        start: isize,
        end: isize,
        stride: Option<isize>,
    ) -> UrlBuilder {
        UrlBuilder {
            inner: self.inner.add_range(variable, start, end, stride),
        }
    }

    #[wasm_bindgen(js_name = addIndexConstraint)]
    pub fn add_index_constraint(self, variable: &str, indices: Box<[IndexRange]>) -> UrlBuilder {
        let rust_indices: Vec<RustIndexRange> = indices.iter().map(|idx| idx.into()).collect();
        UrlBuilder {
            inner: self.inner.add_index_constraint(variable, rust_indices),
        }
    }

    #[wasm_bindgen(js_name = clearVariables)]
    pub fn clear_variables(self) -> UrlBuilder {
        UrlBuilder {
            inner: self.inner.clear_variables(),
        }
    }

    #[wasm_bindgen(js_name = clearConstraints)]
    pub fn clear_constraints(self) -> UrlBuilder {
        UrlBuilder {
            inner: self.inner.clear_constraints(),
        }
    }

    #[wasm_bindgen(js_name = clearAll)]
    pub fn clear_all(self) -> UrlBuilder {
        UrlBuilder {
            inner: self.inner.clear_all(),
        }
    }
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

fn data_value_to_js_value(value: &DataValue) -> JsValue {
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

fn das_attribute_to_js_object(attribute: &DasAttribute) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    Reflect::set(&obj, &"dataType".into(), &data_type_to_string(&attribute.data_type).into())?;
    Reflect::set(&obj, &"name".into(), &attribute.name.clone().into())?;
    Reflect::set(&obj, &"value".into(), &data_value_to_js_value(&attribute.value))?;
    
    Ok(obj.into())
}

fn das_variable_to_js_object(variable: &DasVariable) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    for (name, attribute) in variable.iter() {
        let attr_obj = das_attribute_to_js_object(attribute)?;
        Reflect::set(&obj, &name.clone().into(), &attr_obj)?;
    }
    
    Ok(obj.into())
}

fn das_attributes_to_js_object(attributes: &DasAttributes) -> Result<JsValue, JsValue> {
    let obj = Object::new();
    
    for (name, variable) in attributes.iter() {
        let var_obj = das_variable_to_js_object(variable)?;
        Reflect::set(&obj, &name.clone().into(), &var_obj)?;
    }
    
    Ok(obj.into())
}

#[wasm_bindgen(js_name = parseDasAttributes)]
pub fn parse_das_attributes_js(input: &str) -> Result<JsValue, String> {
    match parse_das_attributes(input) {
        Ok(attributes) => das_attributes_to_js_object(&attributes)
            .map_err(|e| format!("Error converting to JavaScript object: {:?}", e)),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}
