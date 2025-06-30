use readap::url::{
    ConstraintBuilder as CoreConstraintBuilder, IndexSelection,
    OpenDAPUrlBuilder as CoreUrlBuilder, ValueSelection,
};
use wasm_bindgen::prelude::*;
// Remove unused serde imports for now
use js_sys::{Object, Reflect};
use std::collections::HashMap;

/// WASM-exposed OpenDAP URL builder
#[wasm_bindgen]
pub struct OpenDAPUrlBuilder {
    inner: CoreUrlBuilder,
}

#[wasm_bindgen]
impl OpenDAPUrlBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: &str) -> OpenDAPUrlBuilder {
        OpenDAPUrlBuilder {
            inner: CoreUrlBuilder::new(base_url),
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
    pub fn dods_url(&self, constraints: Option<String>) -> String {
        self.inner.dods_url(constraints.as_deref())
    }

    #[wasm_bindgen(js_name = dodsUrlWithConstraints)]
    pub fn dods_url_with_constraints(&self, builder: &ConstraintBuilder) -> String {
        let constraint_str = builder.inner.build();
        self.inner.dods_url(if constraint_str.is_empty() {
            None
        } else {
            Some(&constraint_str)
        })
    }

    #[wasm_bindgen(js_name = baseUrl)]
    pub fn base_url(&self) -> String {
        self.inner.base_url().to_string()
    }
}

/// WASM-exposed constraint builder for creating OpenDAP selection constraints
#[wasm_bindgen]
#[derive(Clone)]
pub struct ConstraintBuilder {
    pub(crate) inner: CoreConstraintBuilder,
}

#[wasm_bindgen]
impl ConstraintBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ConstraintBuilder {
        ConstraintBuilder {
            inner: CoreConstraintBuilder::new(),
        }
    }

    /// Index-based selection (isel style) using a JavaScript object
    ///
    /// Example JS usage:
    /// ```js
    /// builder.isel({
    ///   temperature: { type: "single", value: 5 },
    ///   pressure: { type: "range", start: 0, end: 10 },
    ///   wind: { type: "stride", start: 0, stride: 2, end: 20 },
    ///   locations: { type: "multiple", values: [0, 5, 10] }
    /// });
    /// ```
    #[wasm_bindgen]
    pub fn isel(mut self, selections: &JsValue) -> Result<ConstraintBuilder, JsValue> {
        let selections_map = parse_index_selections(selections)?;
        self.inner = self.inner.isel(selections_map);
        Ok(self)
    }

    /// Value-based selection (sel style) using a JavaScript object
    ///
    /// Example JS usage:
    /// ```js
    /// builder.sel({
    ///   temperature: { type: "single", value: 25.5 },
    ///   pressure: { type: "range", min: 1000.0, max: 500.0 },
    ///   time: { type: "string", value: "2023-01-15" },
    ///   locations: { type: "multiple", values: [40.5, 41.0, 41.5] }
    /// });
    /// ```
    #[wasm_bindgen]
    pub fn sel(mut self, selections: &JsValue) -> Result<ConstraintBuilder, JsValue> {
        let selections_map = parse_value_selections(selections)?;
        self.inner = self.inner.sel(selections_map);
        Ok(self)
    }

    /// Build the constraint string
    #[wasm_bindgen]
    pub fn build(&self) -> String {
        self.inner.build()
    }

    /// Get the constraint information as JSON for debugging
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        // Convert constraints to a serializable format
        let constraints: Vec<_> = self
            .inner
            .constraints()
            .iter()
            .map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "dimensions": c.dimensions.len()
                })
            })
            .collect();

        serde_json::to_string(&constraints).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Parse JavaScript index selection object into Rust HashMap
fn parse_index_selections(js_value: &JsValue) -> Result<HashMap<String, IndexSelection>, JsValue> {
    let mut selections = HashMap::new();

    if !js_value.is_object() {
        return Err(JsValue::from_str("Index selections must be an object"));
    }

    let obj = js_value.dyn_ref::<Object>().unwrap();
    let entries = Object::entries(obj);

    for i in 0..entries.length() {
        let entry = entries.get(i);
        let entry_array = entry.dyn_ref::<js_sys::Array>().unwrap();
        let key = entry_array.get(0).as_string().unwrap();
        let value = entry_array.get(1);

        let selection = parse_single_index_selection(&value)?;
        selections.insert(key, selection);
    }

    Ok(selections)
}

fn parse_single_index_selection(js_value: &JsValue) -> Result<IndexSelection, JsValue> {
    if js_value.is_object() {
        let obj = js_value.dyn_ref::<Object>().unwrap();
        let selection_type = Reflect::get(obj, &JsValue::from_str("type"))?
            .as_string()
            .ok_or_else(|| JsValue::from_str("Missing 'type' field in index selection"))?;

        match selection_type.as_str() {
            "single" => {
                let value = Reflect::get(obj, &JsValue::from_str("value"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'value' field"))?
                    as usize;
                Ok(IndexSelection::Single(value))
            }
            "range" => {
                let start = Reflect::get(obj, &JsValue::from_str("start"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'start' field"))?
                    as usize;
                let end = Reflect::get(obj, &JsValue::from_str("end"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'end' field"))?
                    as usize;
                Ok(IndexSelection::Range(start, end))
            }
            "stride" => {
                let start = Reflect::get(obj, &JsValue::from_str("start"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'start' field"))?
                    as usize;
                let stride = Reflect::get(obj, &JsValue::from_str("stride"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'stride' field"))?
                    as usize;
                let end = Reflect::get(obj, &JsValue::from_str("end"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'end' field"))?
                    as usize;
                Ok(IndexSelection::Stride(start, stride, end))
            }
            "multiple" => {
                let values_js = Reflect::get(obj, &JsValue::from_str("values"))?;
                let values_array = values_js
                    .dyn_ref::<js_sys::Array>()
                    .ok_or_else(|| JsValue::from_str("'values' must be an array"))?;

                let mut values = Vec::new();
                for i in 0..values_array.length() {
                    let val = values_array
                        .get(i)
                        .as_f64()
                        .ok_or_else(|| JsValue::from_str("Array values must be numbers"))?
                        as usize;
                    values.push(val);
                }
                Ok(IndexSelection::Multiple(values))
            }
            _ => Err(JsValue::from_str(&format!(
                "Unknown index selection type: {}",
                selection_type
            ))),
        }
    } else if js_value.is_bigint() || js_value.as_f64().is_some() {
        // Direct number means single index
        let value = js_value.as_f64().unwrap() as usize;
        Ok(IndexSelection::Single(value))
    } else {
        Err(JsValue::from_str("Invalid index selection format"))
    }
}

/// Parse JavaScript value selection object into Rust HashMap
fn parse_value_selections(js_value: &JsValue) -> Result<HashMap<String, ValueSelection>, JsValue> {
    let mut selections = HashMap::new();

    if !js_value.is_object() {
        return Err(JsValue::from_str("Value selections must be an object"));
    }

    let obj = js_value.dyn_ref::<Object>().unwrap();
    let entries = Object::entries(obj);

    for i in 0..entries.length() {
        let entry = entries.get(i);
        let entry_array = entry.dyn_ref::<js_sys::Array>().unwrap();
        let key = entry_array.get(0).as_string().unwrap();
        let value = entry_array.get(1);

        let selection = parse_single_value_selection(&value)?;
        selections.insert(key, selection);
    }

    Ok(selections)
}

fn parse_single_value_selection(js_value: &JsValue) -> Result<ValueSelection, JsValue> {
    if js_value.is_object() {
        let obj = js_value.dyn_ref::<Object>().unwrap();
        let selection_type = Reflect::get(obj, &JsValue::from_str("type"))?
            .as_string()
            .ok_or_else(|| JsValue::from_str("Missing 'type' field in value selection"))?;

        match selection_type.as_str() {
            "single" => {
                let value = Reflect::get(obj, &JsValue::from_str("value"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'value' field"))?;
                Ok(ValueSelection::Single(value))
            }
            "range" => {
                let min = Reflect::get(obj, &JsValue::from_str("min"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'min' field"))?;
                let max = Reflect::get(obj, &JsValue::from_str("max"))?
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'max' field"))?;
                Ok(ValueSelection::Range(min, max))
            }
            "string" => {
                let value = Reflect::get(obj, &JsValue::from_str("value"))?
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'value' field"))?;
                Ok(ValueSelection::String(value))
            }
            "string_range" => {
                let start = Reflect::get(obj, &JsValue::from_str("start"))?
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'start' field"))?;
                let end = Reflect::get(obj, &JsValue::from_str("end"))?
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Missing or invalid 'end' field"))?;
                Ok(ValueSelection::StringRange(start, end))
            }
            "multiple" => {
                let values_js = Reflect::get(obj, &JsValue::from_str("values"))?;
                let values_array = values_js
                    .dyn_ref::<js_sys::Array>()
                    .ok_or_else(|| JsValue::from_str("'values' must be an array"))?;

                let mut values = Vec::new();
                for i in 0..values_array.length() {
                    let val = values_array
                        .get(i)
                        .as_f64()
                        .ok_or_else(|| JsValue::from_str("Array values must be numbers"))?;
                    values.push(val);
                }
                Ok(ValueSelection::Multiple(values))
            }
            "string_multiple" => {
                let values_js = Reflect::get(obj, &JsValue::from_str("values"))?;
                let values_array = values_js
                    .dyn_ref::<js_sys::Array>()
                    .ok_or_else(|| JsValue::from_str("'values' must be an array"))?;

                let mut values = Vec::new();
                for i in 0..values_array.length() {
                    let val = values_array
                        .get(i)
                        .as_string()
                        .ok_or_else(|| JsValue::from_str("Array values must be strings"))?;
                    values.push(val);
                }
                Ok(ValueSelection::StringMultiple(values))
            }
            _ => Err(JsValue::from_str(&format!(
                "Unknown value selection type: {}",
                selection_type
            ))),
        }
    } else if let Some(num_val) = js_value.as_f64() {
        // Direct number means single value
        Ok(ValueSelection::Single(num_val))
    } else if let Some(str_val) = js_value.as_string() {
        // Direct string means single string value
        Ok(ValueSelection::String(str_val))
    } else if js_value.is_array() {
        // Direct array - determine if numbers or strings
        let array = js_value.dyn_ref::<js_sys::Array>().unwrap();
        if array.length() == 0 {
            return Err(JsValue::from_str("Empty array not allowed"));
        }

        let first_element = array.get(0);
        if first_element.as_f64().is_some() {
            // Array of numbers
            if array.length() == 2 {
                // Treat as range [min, max]
                let min = array.get(0).as_f64().unwrap();
                let max = array.get(1).as_f64().unwrap();
                Ok(ValueSelection::Range(min, max))
            } else {
                // Treat as multiple values
                let mut values = Vec::new();
                for i in 0..array.length() {
                    let val = array
                        .get(i)
                        .as_f64()
                        .ok_or_else(|| JsValue::from_str("All array elements must be numbers"))?;
                    values.push(val);
                }
                Ok(ValueSelection::Multiple(values))
            }
        } else if first_element.as_string().is_some() {
            // Array of strings
            if array.length() == 2 {
                // Treat as string range
                let start = array.get(0).as_string().unwrap();
                let end = array.get(1).as_string().unwrap();
                Ok(ValueSelection::StringRange(start, end))
            } else {
                // Treat as multiple string values
                let mut values = Vec::new();
                for i in 0..array.length() {
                    let val = array
                        .get(i)
                        .as_string()
                        .ok_or_else(|| JsValue::from_str("All array elements must be strings"))?;
                    values.push(val);
                }
                Ok(ValueSelection::StringMultiple(values))
            }
        } else {
            Err(JsValue::from_str(
                "Array elements must be either all numbers or all strings",
            ))
        }
    } else {
        Err(JsValue::from_str("Invalid value selection format"))
    }
}
