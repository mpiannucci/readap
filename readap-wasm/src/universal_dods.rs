/// Universal DODS parser that works across all JavaScript runtimes
/// This provides a more robust DODS parsing implementation that handles
/// runtime-specific issues and provides better error reporting

use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use std::collections::HashMap;

/// Result of DODS parsing with detailed error information
#[wasm_bindgen]
pub struct DodsParseResult {
    success: bool,
    error_message: Option<String>,
    variables: HashMap<String, DodsVariable>,
}

/// Represents a parsed variable from DODS data
#[derive(Debug, Clone)]
pub struct DodsVariable {
    name: String,
    data_type: String,
    values: Vec<f64>, // Normalized to f64 for simplicity
    dimensions: Vec<usize>,
}

/// Universal DODS parser that handles different runtime environments
#[wasm_bindgen]
pub struct UniversalDodsParser {
    debug_mode: bool,
}

#[wasm_bindgen]
impl UniversalDodsParser {
    /// Create a new universal DODS parser
    #[wasm_bindgen(constructor)]
    pub fn new() -> UniversalDodsParser {
        UniversalDodsParser {
            debug_mode: false,
        }
    }

    /// Enable debug mode for detailed parsing information
    #[wasm_bindgen(js_name = setDebugMode)]
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }

    /// Parse DODS binary data with improved error handling
    #[wasm_bindgen(js_name = parseDods)]
    pub fn parse_dods(&self, data: &Uint8Array) -> Result<Object, JsValue> {
        let bytes = data.to_vec();
        
        if self.debug_mode {
            web_sys::console::log_1(&format!("Parsing {} bytes of DODS data", bytes.len()).into());
        }

        match self.parse_dods_internal(&bytes) {
            Ok(result) => self.convert_result_to_js(result),
            Err(e) => Err(JsValue::from_str(&format!("DODS parsing failed: {}", e))),
        }
    }

    /// Parse DODS data and return detailed result information
    #[wasm_bindgen(js_name = parseDodsDetailed)]
    pub fn parse_dods_detailed(&self, data: &Uint8Array) -> Object {
        let bytes = data.to_vec();
        let result = self.parse_dods_internal(&bytes);
        
        let js_result = Object::new();
        
        match result {
            Ok(variables) => {
                Reflect::set(&js_result, &JsValue::from_str("success"), &JsValue::from_bool(true)).unwrap();
                Reflect::set(&js_result, &JsValue::from_str("variables"), &self.convert_variables_to_js(&variables)).unwrap();
            },
            Err(e) => {
                Reflect::set(&js_result, &JsValue::from_str("success"), &JsValue::from_bool(false)).unwrap();
                Reflect::set(&js_result, &JsValue::from_str("error"), &JsValue::from_str(&e)).unwrap();
            }
        }
        
        js_result
    }

    /// Analyze DODS data structure without full parsing
    #[wasm_bindgen(js_name = analyzeDodsStructure)]
    pub fn analyze_dods_structure(&self, data: &Uint8Array) -> Object {
        let bytes = data.to_vec();
        let analysis = Object::new();
        
        // Convert to string to find the Data: marker
        let text = String::from_utf8_lossy(&bytes);
        
        // Find Data: marker
        let data_marker = text.find("Data:");
        Reflect::set(&analysis, &JsValue::from_str("hasDataMarker"), &JsValue::from_bool(data_marker.is_some())).unwrap();
        
        if let Some(marker_pos) = data_marker {
            Reflect::set(&analysis, &JsValue::from_str("dataMarkerPosition"), &JsValue::from_f64(marker_pos as f64)).unwrap();
            
            // Calculate binary data position - find actual start after whitespace
            let mut binary_start = marker_pos + 5; // Start after "Data:"
            while binary_start < bytes.len() && (bytes[binary_start] == b'\r' || bytes[binary_start] == b'\n' || bytes[binary_start] == b' ') {
                binary_start += 1;
            }
            let binary_length = bytes.len() - binary_start;
            
            Reflect::set(&analysis, &JsValue::from_str("binaryDataStart"), &JsValue::from_f64(binary_start as f64)).unwrap();
            Reflect::set(&analysis, &JsValue::from_str("binaryDataLength"), &JsValue::from_f64(binary_length as f64)).unwrap();
            
            // Analyze binary data structure
            if binary_length >= 8 {
                let binary_data = &bytes[binary_start..];
                let analysis_result = self.analyze_binary_structure(binary_data);
                Reflect::set(&analysis, &JsValue::from_str("binaryAnalysis"), &analysis_result).unwrap();
            }
        }
        
        Reflect::set(&analysis, &JsValue::from_str("totalSize"), &JsValue::from_f64(bytes.len() as f64)).unwrap();
        
        analysis
    }
}

impl UniversalDodsParser {
    /// Internal DODS parsing implementation
    fn parse_dods_internal(&self, bytes: &[u8]) -> Result<HashMap<String, DodsVariable>, String> {
        // Convert to string to find metadata
        let text = String::from_utf8_lossy(bytes);
        
        // Find the Data: marker
        let data_marker = text.find("Data:")
            .ok_or("No 'Data:' marker found in DODS response")?;
        
        if self.debug_mode {
            web_sys::console::log_1(&format!("Found Data: marker at position {}", data_marker).into());
        }

        // Parse the DDS portion (before Data:)
        let dds_text = &text[..data_marker];
        let variable_info = self.parse_dds_info(dds_text)?;
        
        // Binary data starts after "Data:\n" - need to find the actual newline
        let mut binary_start = data_marker + 5; // Start after "Data:"
        
        // Find the actual newline character(s)
        while binary_start < bytes.len() && (bytes[binary_start] == b'\r' || bytes[binary_start] == b'\n' || bytes[binary_start] == b' ') {
            binary_start += 1;
        }
        
        if binary_start >= bytes.len() {
            return Err("No binary data found after Data: marker".to_string());
        }
        
        let binary_data = &bytes[binary_start..];
        
        if self.debug_mode {
            web_sys::console::log_1(&format!("Binary data length: {} bytes", binary_data.len()).into());
            if binary_data.len() >= 16 {
                let hex_preview: Vec<String> = binary_data.iter().take(16).map(|b| format!("{:02x}", b)).collect();
                web_sys::console::log_1(&format!("First 16 bytes: {}", hex_preview.join(" ")).into());
            }
        }
        
        // Parse binary data for each variable
        self.parse_binary_data(binary_data, variable_info)
    }

    /// Extract variable information from DDS text
    fn parse_dds_info(&self, dds_text: &str) -> Result<Vec<(String, String, Vec<usize>)>, String> {
        let mut variables = Vec::new();
        
        // Simple DDS parsing - look for Array declarations
        for line in dds_text.lines() {
            let trimmed = line.trim();
            
            // Look for patterns like: Float32 t2m[longitude = 1][latitude = 1]...
            if trimmed.contains("[") && (trimmed.contains("Float32") || trimmed.contains("Float64") || trimmed.contains("Int32")) {
                if let Some(var_info) = self.parse_variable_declaration(trimmed) {
                    variables.push(var_info);
                }
            }
        }
        
        if variables.is_empty() {
            return Err("No variables found in DDS".to_string());
        }
        
        Ok(variables)
    }

    /// Parse a single variable declaration line
    fn parse_variable_declaration(&self, line: &str) -> Option<(String, String, Vec<usize>)> {
        // Extract data type
        let data_type = if line.contains("Float32") {
            "Float32"
        } else if line.contains("Float64") {
            "Float64"
        } else if line.contains("Int32") {
            "Int32"
        } else {
            return None;
        };

        // Extract variable name and dimensions
        let parts: Vec<&str> = line.split_whitespace().collect();
        for i in 0..parts.len() {
            if parts[i] == data_type && i + 1 < parts.len() {
                let var_decl = parts[i + 1];
                
                // Split on '[' to get variable name
                if let Some(bracket_pos) = var_decl.find('[') {
                    let var_name = var_decl[..bracket_pos].to_string();
                    
                    // Extract dimensions - simplified parsing
                    let mut dimensions = Vec::new();
                    let mut current_pos = bracket_pos;
                    
                    while let Some(start) = var_decl[current_pos..].find("= ") {
                        let start_pos = current_pos + start + 2;
                        if let Some(end) = var_decl[start_pos..].find(']') {
                            let end_pos = start_pos + end;
                            if let Ok(size) = var_decl[start_pos..end_pos].parse::<usize>() {
                                dimensions.push(size);
                            }
                            current_pos = end_pos + 1;
                        } else {
                            break;
                        }
                    }
                    
                    return Some((var_name, data_type.to_string(), dimensions));
                }
            }
        }
        
        None
    }

    /// Parse binary data section
    fn parse_binary_data(&self, binary_data: &[u8], variables: Vec<(String, String, Vec<usize>)>) -> Result<HashMap<String, DodsVariable>, String> {
        let mut result = HashMap::new();
        let mut offset = 0;
        
        for (var_name, data_type, dimensions) in variables {
            if offset >= binary_data.len() {
                break;
            }
            
            if self.debug_mode {
                web_sys::console::log_1(&format!("Parsing variable {} at offset {}", var_name, offset).into());
            }
            
            match self.parse_variable_data(&binary_data[offset..], &data_type, &dimensions) {
                Ok((values, bytes_consumed)) => {
                    let variable = DodsVariable {
                        name: var_name.clone(),
                        data_type: data_type.clone(),
                        values,
                        dimensions: dimensions.clone(),
                    };
                    result.insert(var_name, variable);
                    offset += bytes_consumed;
                }
                Err(e) => {
                    if self.debug_mode {
                        web_sys::console::log_1(&format!("Failed to parse variable {}: {}", var_name, e).into());
                    }
                    return Err(format!("Failed to parse variable {}: {}", var_name, e));
                }
            }
        }
        
        Ok(result)
    }

    /// Parse data for a single variable
    fn parse_variable_data(&self, data: &[u8], data_type: &str, dimensions: &[usize]) -> Result<(Vec<f64>, usize), String> {
        if data.len() < 8 {
            return Err("Insufficient data for count headers".to_string());
        }
        
        // Read count (appears twice in OpenDAP format)
        let count1 = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let count2 = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        
        if self.debug_mode {
            web_sys::console::log_1(&format!("Counts: {} and {} (match: {})", count1, count2, count1 == count2).into());
        }
        
        if count1 != count2 {
            return Err(format!("Count mismatch: {} != {}", count1, count2));
        }
        
        let element_count = count1 as usize;
        let mut values = Vec::new();
        let mut offset = 8; // Skip the two count fields
        
        // Calculate expected element count from dimensions
        let expected_count = dimensions.iter().product::<usize>();
        if element_count != expected_count {
            if self.debug_mode {
                web_sys::console::log_1(&format!("Count mismatch: got {}, expected {} from dimensions {:?}", element_count, expected_count, dimensions).into());
            }
        }
        
        // Parse data values based on type
        for _ in 0..element_count {
            if offset >= data.len() {
                return Err("Insufficient data for all elements".to_string());
            }
            
            let value = match data_type {
                "Float32" => {
                    if offset + 4 > data.len() {
                        return Err("Insufficient data for Float32".to_string());
                    }
                    let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
                    let value = f32::from_be_bytes(bytes) as f64;
                    offset += 4;
                    value
                }
                "Float64" => {
                    if offset + 8 > data.len() {
                        return Err("Insufficient data for Float64".to_string());
                    }
                    let bytes = [
                        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                        data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
                    ];
                    let value = f64::from_be_bytes(bytes);
                    offset += 8;
                    value
                }
                "Int32" => {
                    if offset + 4 > data.len() {
                        return Err("Insufficient data for Int32".to_string());
                    }
                    let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
                    let value = i32::from_be_bytes(bytes) as f64;
                    offset += 4;
                    value
                }
                _ => {
                    return Err(format!("Unsupported data type: {}", data_type));
                }
            };
            
            values.push(value);
        }
        
        Ok((values, offset))
    }

    /// Analyze binary data structure for debugging
    fn analyze_binary_structure(&self, binary_data: &[u8]) -> Object {
        let analysis = Object::new();
        
        if binary_data.len() >= 8 {
            let count1 = u32::from_be_bytes([binary_data[0], binary_data[1], binary_data[2], binary_data[3]]);
            let count2 = u32::from_be_bytes([binary_data[4], binary_data[5], binary_data[6], binary_data[7]]);
            
            Reflect::set(&analysis, &JsValue::from_str("count1"), &JsValue::from_f64(count1 as f64)).unwrap();
            Reflect::set(&analysis, &JsValue::from_str("count2"), &JsValue::from_f64(count2 as f64)).unwrap();
            Reflect::set(&analysis, &JsValue::from_str("countsMatch"), &JsValue::from_bool(count1 == count2)).unwrap();
            
            if count1 == count2 && count1 > 0 && binary_data.len() >= 12 {
                // Try to read first data value as Float32
                let float_bytes = [binary_data[8], binary_data[9], binary_data[10], binary_data[11]];
                let float_value = f32::from_be_bytes(float_bytes);
                Reflect::set(&analysis, &JsValue::from_str("firstFloat32"), &JsValue::from_f64(float_value as f64)).unwrap();
            }
        }
        
        // Show first 16 bytes as hex
        let hex_bytes: Vec<String> = binary_data.iter().take(16).map(|b| format!("{:02x}", b)).collect();
        let hex_string = hex_bytes.join(" ");
        Reflect::set(&analysis, &JsValue::from_str("hexPreview"), &JsValue::from_str(&hex_string)).unwrap();
        
        analysis
    }

    /// Convert parsing result to JavaScript object
    fn convert_result_to_js(&self, variables: HashMap<String, DodsVariable>) -> Result<Object, JsValue> {
        let result = Object::new();
        
        for (name, variable) in variables {
            let var_obj = Object::new();
            
            Reflect::set(&var_obj, &JsValue::from_str("name"), &JsValue::from_str(&variable.name))?;
            Reflect::set(&var_obj, &JsValue::from_str("type"), &JsValue::from_str(&variable.data_type))?;
            Reflect::set(&var_obj, &JsValue::from_str("length"), &JsValue::from_f64(variable.values.len() as f64))?;
            
            // Convert values to JavaScript array
            let js_array = js_sys::Float64Array::new_with_length(variable.values.len() as u32);
            for (i, &value) in variable.values.iter().enumerate() {
                js_array.set_index(i as u32, value);
            }
            Reflect::set(&var_obj, &JsValue::from_str("data"), &js_array)?;
            
            // Convert dimensions to JavaScript array
            let dims_array = Array::new();
            for &dim in &variable.dimensions {
                dims_array.push(&JsValue::from_f64(dim as f64));
            }
            Reflect::set(&var_obj, &JsValue::from_str("dimensions"), &dims_array)?;
            
            Reflect::set(&result, &JsValue::from_str(&name), &var_obj)?;
        }
        
        Ok(result)
    }

    /// Convert variables map to JavaScript object
    fn convert_variables_to_js(&self, variables: &HashMap<String, DodsVariable>) -> Object {
        let result = Object::new();
        
        for (name, variable) in variables {
            let var_obj = Object::new();
            
            Reflect::set(&var_obj, &JsValue::from_str("name"), &JsValue::from_str(&variable.name)).unwrap();
            Reflect::set(&var_obj, &JsValue::from_str("type"), &JsValue::from_str(&variable.data_type)).unwrap();
            Reflect::set(&var_obj, &JsValue::from_str("valueCount"), &JsValue::from_f64(variable.values.len() as f64)).unwrap();
            
            let dims_array = Array::new();
            for &dim in &variable.dimensions {
                dims_array.push(&JsValue::from_f64(dim as f64));
            }
            Reflect::set(&var_obj, &JsValue::from_str("dimensions"), &dims_array).unwrap();
            
            Reflect::set(&result, &JsValue::from_str(name), &var_obj).unwrap();
        }
        
        result
    }
}

/// Helper function to create a universal DODS parser
#[wasm_bindgen(js_name = createUniversalDodsParser)]
pub fn create_universal_dods_parser() -> UniversalDodsParser {
    UniversalDodsParser::new()
}