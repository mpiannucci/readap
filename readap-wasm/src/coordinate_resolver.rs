use crate::url_builder::ConstraintBuilder;
use js_sys::{Array, Float64Array};
use readap::url::CoordinateResolver as CoreCoordinateResolver;
use wasm_bindgen::prelude::*;

/// WASM-exposed coordinate resolver for converting value-based selections to index-based selections
#[wasm_bindgen]
pub struct CoordinateResolver {
    inner: CoreCoordinateResolver,
}

#[wasm_bindgen]
impl CoordinateResolver {
    #[wasm_bindgen(constructor)]
    pub fn new() -> CoordinateResolver {
        CoordinateResolver {
            inner: CoreCoordinateResolver::new(),
        }
    }

    /// Add coordinate data for a variable using a Float64Array for efficient transfer
    #[wasm_bindgen(js_name = addCoordinates)]
    pub fn add_coordinates(
        &mut self,
        var_name: &str,
        coords: &Float64Array,
    ) -> Result<(), JsValue> {
        let coords_vec: Vec<f64> = coords.to_vec();
        self.inner.add_coordinates(var_name.to_string(), coords_vec);
        Ok(())
    }

    /// Add coordinate data using a regular JavaScript array (less efficient but more flexible)
    #[wasm_bindgen(js_name = addCoordinatesFromArray)]
    pub fn add_coordinates_from_array(
        &mut self,
        var_name: &str,
        coords: &Array,
    ) -> Result<(), JsValue> {
        let mut coords_vec = Vec::with_capacity(coords.length() as usize);

        for i in 0..coords.length() {
            let val = coords
                .get(i)
                .as_f64()
                .ok_or_else(|| JsValue::from_str("All coordinate values must be numbers"))?;
            coords_vec.push(val);
        }

        self.inner.add_coordinates(var_name.to_string(), coords_vec);
        Ok(())
    }

    /// Resolve value-based constraints to index-based constraints using nearest neighbor lookup
    #[wasm_bindgen(js_name = resolveConstraints)]
    pub fn resolve_constraints(
        &self,
        builder: &ConstraintBuilder,
    ) -> Result<ConstraintBuilder, JsValue> {
        let resolved_core = self
            .inner
            .resolve_constraints(&builder.inner)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(ConstraintBuilder {
            inner: resolved_core,
        })
    }

    /// Get information about cached coordinates for debugging
    #[wasm_bindgen(js_name = getCoordinateInfo)]
    pub fn get_coordinate_info(&self) -> Result<String, JsValue> {
        // Since coordinate_cache is private, we'll need to modify the core to expose this
        // For now, return a placeholder
        Ok("Coordinate resolver ready".to_string())
    }
}

/// Utility functions for coordinate operations
#[wasm_bindgen]
pub struct CoordinateUtils;

#[wasm_bindgen]
impl CoordinateUtils {
    /// Find the nearest index for a given coordinate value using binary search
    /// This is exposed for advanced users who want to do their own coordinate lookups
    #[wasm_bindgen(js_name = findNearestIndex)]
    pub fn find_nearest_index(coords: &Float64Array, target: f64) -> Result<usize, JsValue> {
        let coords_vec = coords.to_vec();
        readap::url::find_nearest_index(&coords_vec, target).map_err(|e| JsValue::from_str(&e))
    }

    /// Find the nearest index using a JavaScript array (less efficient)
    #[wasm_bindgen(js_name = findNearestIndexFromArray)]
    pub fn find_nearest_index_from_array(coords: &Array, target: f64) -> Result<usize, JsValue> {
        let mut coords_vec = Vec::with_capacity(coords.length() as usize);

        for i in 0..coords.length() {
            let val = coords
                .get(i)
                .as_f64()
                .ok_or_else(|| JsValue::from_str("All coordinate values must be numbers"))?;
            coords_vec.push(val);
        }

        readap::url::find_nearest_index(&coords_vec, target).map_err(|e| JsValue::from_str(&e))
    }

    /// Create evenly spaced coordinates (like numpy.linspace)
    #[wasm_bindgen(js_name = linspace)]
    pub fn linspace(start: f64, end: f64, num: usize) -> Float64Array {
        if num == 0 {
            return Float64Array::new_with_length(0);
        }

        if num == 1 {
            let result = Float64Array::new_with_length(1);
            result.set_index(0, start);
            return result;
        }

        let step = (end - start) / (num - 1) as f64;
        let result = Float64Array::new_with_length(num as u32);

        for i in 0..num {
            let value = start + (i as f64) * step;
            result.set_index(i as u32, value);
        }

        result
    }

    /// Create coordinates from a range with step (like numpy.arange)
    #[wasm_bindgen(js_name = arange)]
    pub fn arange(start: f64, end: f64, step: f64) -> Result<Float64Array, JsValue> {
        if step == 0.0 {
            return Err(JsValue::from_str("Step cannot be zero"));
        }

        if (step > 0.0 && start >= end) || (step < 0.0 && start <= end) {
            return Ok(Float64Array::new_with_length(0));
        }

        let num_elements = ((end - start) / step).abs().ceil() as usize;
        let result = Float64Array::new_with_length(num_elements as u32);

        let mut current = start;
        let mut i = 0;

        while (step > 0.0 && current < end) || (step < 0.0 && current > end) {
            if i >= num_elements {
                break;
            }
            result.set_index(i as u32, current);
            current += step;
            i += 1;
        }

        // Resize if we ended up with fewer elements
        if i < num_elements {
            let final_result = Float64Array::new_with_length(i as u32);
            for j in 0..i {
                final_result.set_index(j as u32, result.get_index(j as u32));
            }
            return Ok(final_result);
        }

        Ok(result)
    }
}
