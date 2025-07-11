use readap::dds::{DdsArray, DdsDataset, DdsGrid, DdsValue};
use wasm_bindgen::prelude::*;
use crate::converters::{dds_array_to_js_object, dds_grid_to_js_object, dds_structure_to_js_object, dds_sequence_to_js_object};

#[wasm_bindgen]
pub struct DdsDatasetWrapper {
    dataset: DdsDataset,
}

#[wasm_bindgen]
impl DdsDatasetWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(dds_string: &str) -> Result<DdsDatasetWrapper, String> {
        match DdsDataset::from_bytes(dds_string) {
            Ok(dataset) => Ok(DdsDatasetWrapper { dataset }),
            Err(e) => Err(format!("Parse error: {}", e)),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.dataset.name.clone()
    }

    #[wasm_bindgen(js_name = listVariables)]
    pub fn list_variables(&self) -> Vec<String> {
        self.dataset.list_variables()
    }

    #[wasm_bindgen(js_name = listCoordinates)]
    pub fn list_coordinates(&self) -> Vec<String> {
        self.dataset.list_coordinates()
    }

    #[wasm_bindgen(js_name = hasVariable)]
    pub fn has_variable(&self, name: &str) -> bool {
        self.dataset.has_variable(name)
    }

    #[wasm_bindgen(js_name = hasCoordinate)]
    pub fn has_coordinate(&self, name: &str) -> bool {
        self.dataset.has_coordinate(name)
    }

    #[wasm_bindgen(js_name = getVariableInfo)]
    pub fn get_variable_info(&self, name: &str) -> JsValue {
        match self.dataset.get_variable_info(name) {
            Some(info) => {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"name".into(), &info.name.into()).unwrap();
                js_sys::Reflect::set(&obj, &"dataType".into(), &crate::converters::data_type_to_string(&info.data_type).into()).unwrap();
                js_sys::Reflect::set(&obj, &"variableType".into(), &format!("{:?}", info.variable_type).into()).unwrap();
                
                let coords_array = js_sys::Array::new();
                for coord in &info.coordinates {
                    coords_array.push(&coord.clone().into());
                }
                js_sys::Reflect::set(&obj, &"coordinates".into(), &coords_array.into()).unwrap();
                
                let dims_array = js_sys::Array::new();
                for (dim_name, dim_size) in &info.dimensions {
                    let dim_obj = js_sys::Object::new();
                    js_sys::Reflect::set(&dim_obj, &"name".into(), &dim_name.clone().into()).unwrap();
                    js_sys::Reflect::set(&dim_obj, &"size".into(), &(*dim_size).into()).unwrap();
                    dims_array.push(&dim_obj.into());
                }
                js_sys::Reflect::set(&obj, &"dimensions".into(), &dims_array.into()).unwrap();
                
                obj.into()
            }
            None => JsValue::NULL
        }
    }

    #[wasm_bindgen(js_name = getCoordinateInfo)]
    pub fn get_coordinate_info(&self, name: &str) -> JsValue {
        match self.dataset.get_coordinate_info(name) {
            Some(info) => {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"name".into(), &info.name.into()).unwrap();
                js_sys::Reflect::set(&obj, &"dataType".into(), &crate::converters::data_type_to_string(&info.data_type).into()).unwrap();
                js_sys::Reflect::set(&obj, &"size".into(), &info.size.into()).unwrap();
                
                let vars_array = js_sys::Array::new();
                for var in &info.variables_using {
                    vars_array.push(&var.clone().into());
                }
                js_sys::Reflect::set(&obj, &"variablesUsing".into(), &vars_array.into()).unwrap();
                
                obj.into()
            }
            None => JsValue::NULL
        }
    }

    #[wasm_bindgen(js_name = getVariable)]
    pub fn get_variable(&self, name: &str) -> JsValue {
        for value in &self.dataset.values {
            if value.name() == name {
                return match value {
                    DdsValue::Array(array) => dds_array_to_js_object(array).unwrap_or(JsValue::NULL),
                    DdsValue::Grid(grid) => dds_grid_to_js_object(grid).unwrap_or(JsValue::NULL),
                    DdsValue::Structure(structure) => dds_structure_to_js_object(structure).unwrap_or(JsValue::NULL),
                    DdsValue::Sequence(sequence) => dds_sequence_to_js_object(sequence).unwrap_or(JsValue::NULL),
                };
            }
        }
        JsValue::NULL
    }

    #[wasm_bindgen(js_name = getVariableCount)]
    pub fn get_variable_count(&self) -> usize {
        self.dataset.values.len()
    }

    #[wasm_bindgen(js_name = getVariableAt)]
    pub fn get_variable_at(&self, index: usize) -> JsValue {
        match self.dataset.values.get(index) {
            Some(value) => {
                match value {
                    DdsValue::Array(array) => dds_array_to_js_object(array).unwrap_or(JsValue::NULL),
                    DdsValue::Grid(grid) => dds_grid_to_js_object(grid).unwrap_or(JsValue::NULL),
                    DdsValue::Structure(structure) => dds_structure_to_js_object(structure).unwrap_or(JsValue::NULL),
                    DdsValue::Sequence(sequence) => dds_sequence_to_js_object(sequence).unwrap_or(JsValue::NULL),
                }
            }
            None => JsValue::NULL
        }
    }
}

#[wasm_bindgen]
pub struct DdsArrayWrapper {
    array: DdsArray,
}

#[wasm_bindgen]
impl DdsArrayWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(dds_string: &str) -> Result<DdsArrayWrapper, String> {
        match DdsArray::parse(dds_string) {
            Ok((_, array)) => Ok(DdsArrayWrapper { array }),
            Err(e) => Err(format!("Parse error: {:?}", e)),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.array.name.clone()
    }

    #[wasm_bindgen(js_name = dataType)]
    pub fn data_type(&self) -> String {
        crate::converters::data_type_to_string(&self.array.data_type)
    }

    #[wasm_bindgen(js_name = arrayLength)]
    pub fn array_length(&self) -> u32 {
        self.array.array_length()
    }

    #[wasm_bindgen(js_name = byteCount)]
    pub fn byte_count(&self) -> usize {
        self.array.byte_count()
    }

    #[wasm_bindgen(js_name = getCoordinates)]
    pub fn get_coordinates(&self) -> JsValue {
        let coords_array = js_sys::Array::new();
        for (coord_name, coord_size) in &self.array.coords {
            let coord_obj = js_sys::Object::new();
            js_sys::Reflect::set(&coord_obj, &"name".into(), &coord_name.clone().into()).unwrap();
            js_sys::Reflect::set(&coord_obj, &"size".into(), &(*coord_size).into()).unwrap();
            coords_array.push(&coord_obj.into());
        }
        coords_array.into()
    }

    #[wasm_bindgen(js_name = toJs)]
    pub fn to_js(&self) -> Result<JsValue, String> {
        dds_array_to_js_object(&self.array)
            .map_err(|e| format!("Error converting to JavaScript object: {:?}", e))
    }
}

#[wasm_bindgen]
pub struct DdsGridWrapper {
    grid: DdsGrid,
}

#[wasm_bindgen]
impl DdsGridWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(dds_string: &str) -> Result<DdsGridWrapper, String> {
        match DdsGrid::parse(dds_string) {
            Ok((_, grid)) => Ok(DdsGridWrapper { grid }),
            Err(e) => Err(format!("Parse error: {:?}", e)),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.grid.name.clone()
    }

    #[wasm_bindgen(js_name = byteCount)]
    pub fn byte_count(&self) -> usize {
        self.grid.byte_count()
    }

    #[wasm_bindgen(js_name = coordsOffset)]
    pub fn coords_offset(&self) -> usize {
        self.grid.coords_offset()
    }

    #[wasm_bindgen(js_name = getCoordOffsets)]
    pub fn get_coord_offsets(&self) -> Vec<usize> {
        self.grid.coord_offsets()
    }

    #[wasm_bindgen(js_name = getArray)]
    pub fn get_array(&self) -> Result<JsValue, String> {
        dds_array_to_js_object(&self.grid.array)
            .map_err(|e| format!("Error converting array to JavaScript object: {:?}", e))
    }

    #[wasm_bindgen(js_name = getCoordinates)]
    pub fn get_coordinates(&self) -> Result<JsValue, String> {
        let coords_array = js_sys::Array::new();
        for coord in &self.grid.coords {
            let coord_obj = dds_array_to_js_object(coord)
                .map_err(|e| format!("Error converting coordinate to JavaScript object: {:?}", e))?;
            coords_array.push(&coord_obj);
        }
        Ok(coords_array.into())
    }

    #[wasm_bindgen(js_name = toJs)]
    pub fn to_js(&self) -> Result<JsValue, String> {
        dds_grid_to_js_object(&self.grid)
            .map_err(|e| format!("Error converting to JavaScript object: {:?}", e))
    }
}