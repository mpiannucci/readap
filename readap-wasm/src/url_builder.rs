use wasm_bindgen::prelude::*;
use readap::{UrlBuilder, IndexRange, Constraint};
use js_sys::{Array, Object, Reflect};

#[wasm_bindgen]
pub struct JsUrlBuilder {
    inner: UrlBuilder,
}

#[wasm_bindgen]
impl JsUrlBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: &str) -> JsUrlBuilder {
        JsUrlBuilder { 
            inner: UrlBuilder::new(base_url) 
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
    pub fn dods_url(&self) -> Result<String, JsValue> {
        match self.inner.dods_url() {
            Ok(url) => Ok(url),
            Err(e) => Err(JsValue::from_str(&format!("Error building DODS URL: {}", e))),
        }
    }

    #[wasm_bindgen(js_name = addVariable)]
    pub fn add_variable(self, variable: &str) -> JsUrlBuilder {
        JsUrlBuilder {
            inner: self.inner.add_variable(variable)
        }
    }

    #[wasm_bindgen(js_name = addVariables)]
    pub fn add_variables(self, variables: &Array) -> JsUrlBuilder {
        let vars: Vec<String> = variables
            .iter()
            .map(|v| v.as_string().unwrap_or_default())
            .collect();
        
        JsUrlBuilder {
            inner: self.inner.add_variables(vars)
        }
    }

    #[wasm_bindgen(js_name = addConstraint)]
    pub fn add_constraint(self, variable: &str, indices: &Array) -> Result<JsUrlBuilder, JsValue> {
        let mut index_ranges = Vec::new();
        
        for i in 0..indices.length() {
            let item = indices.get(i);
            
            // Try to parse as object with start/end/stride properties
            if let Ok(obj) = item.clone().dyn_into::<Object>() {
                let start = Reflect::get(&obj, &"start".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as isize);
                
                let end = Reflect::get(&obj, &"end".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as isize);
                
                let stride = Reflect::get(&obj, &"stride".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as isize);
                
                match (start, end) {
                    (Some(start), Some(end)) => {
                        index_ranges.push(IndexRange::Range { start, end, stride });
                    },
                    _ => {
                        // Try as single index
                        if let Some(index) = item.as_f64() {
                            index_ranges.push(IndexRange::Single(index as isize));
                        } else {
                            return Err(JsValue::from_str("Invalid constraint format"));
                        }
                    }
                }
            } else if let Some(index) = item.as_f64() {
                // Single index
                index_ranges.push(IndexRange::Single(index as isize));
            } else {
                return Err(JsValue::from_str("Invalid constraint format"));
            }
        }
        
        let constraint = Constraint::new(variable, index_ranges);
        Ok(JsUrlBuilder {
            inner: self.inner.add_constraint(constraint)
        })
    }

    #[wasm_bindgen(js_name = addSingleIndex)]
    pub fn add_single_index(self, variable: &str, index: isize) -> JsUrlBuilder {
        JsUrlBuilder {
            inner: self.inner.add_single_index(variable, index)
        }
    }

    #[wasm_bindgen(js_name = addRange)]
    pub fn add_range(self, variable: &str, start: isize, end: isize, stride: Option<isize>) -> JsUrlBuilder {
        JsUrlBuilder {
            inner: self.inner.add_range(variable, start, end, stride)
        }
    }

    #[wasm_bindgen(js_name = addMultidimensionalConstraint)]
    pub fn add_multidimensional_constraint(self, variable: &str, indices: &Array) -> Result<JsUrlBuilder, JsValue> {
        let mut index_ranges = Vec::new();
        
        for i in 0..indices.length() {
            let item = indices.get(i);
            
            // Try to parse as object with start/end/stride properties
            if let Ok(obj) = item.clone().dyn_into::<Object>() {
                let start = Reflect::get(&obj, &"start".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as isize);
                
                let end = Reflect::get(&obj, &"end".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as isize);
                
                let stride = Reflect::get(&obj, &"stride".into())
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|v| v as isize);
                
                match (start, end) {
                    (Some(start), Some(end)) => {
                        index_ranges.push(IndexRange::Range { start, end, stride });
                    },
                    _ => {
                        // Try as single index
                        if let Some(index) = item.as_f64() {
                            index_ranges.push(IndexRange::Single(index as isize));
                        } else {
                            return Err(JsValue::from_str("Invalid constraint format"));
                        }
                    }
                }
            } else if let Some(index) = item.as_f64() {
                // Single index
                index_ranges.push(IndexRange::Single(index as isize));
            } else {
                return Err(JsValue::from_str("Invalid constraint format"));
            }
        }
        
        Ok(JsUrlBuilder {
            inner: self.inner.add_multidimensional_constraint(variable, index_ranges)
        })
    }

    #[wasm_bindgen(js_name = clearVariables)]
    pub fn clear_variables(self) -> JsUrlBuilder {
        JsUrlBuilder {
            inner: self.inner.clear_variables()
        }
    }

    #[wasm_bindgen(js_name = clearConstraints)]
    pub fn clear_constraints(self) -> JsUrlBuilder {
        JsUrlBuilder {
            inner: self.inner.clear_constraints()
        }
    }

    #[wasm_bindgen(js_name = clearAll)]
    pub fn clear_all(self) -> JsUrlBuilder {
        JsUrlBuilder {
            inner: self.inner.clear_all()
        }
    }

    #[wasm_bindgen(js_name = clone)]
    pub fn clone_builder(&self) -> JsUrlBuilder {
        JsUrlBuilder {
            inner: self.inner.clone()
        }
    }
}