/// Runtime-agnostic fetch abstraction for readap-wasm
/// Works across Browser, Node.js, Bun, Deno, and other JavaScript runtimes
use js_sys::{ArrayBuffer, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Universal fetch result containing response data and metadata
#[wasm_bindgen]
pub struct FetchResponse {
    pub(crate) status: u16,
    pub(crate) status_text: String,
    pub(crate) data: FetchData,
    pub(crate) headers: Object,
}

/// Represents different types of response data
pub(crate) enum FetchData {
    Text(String),
    Binary(Vec<u8>),
}

/// Universal fetch client that works across all JavaScript runtimes
#[wasm_bindgen]
pub struct UniversalFetch {
    /// Runtime detection results
    runtime_info: RuntimeInfo,
    /// Default fetch options
    default_options: Object,
}

struct RuntimeInfo {
    runtime_type: RuntimeType,
    has_fetch: bool,
    has_request_class: bool,
}

#[derive(Debug, Clone)]
enum RuntimeType {
    Browser,
    NodeJs,
    Bun,
    Deno,
    Unknown,
}

#[wasm_bindgen]
impl UniversalFetch {
    /// Create a new universal fetch client with runtime detection
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<UniversalFetch, JsValue> {
        let runtime_info = Self::detect_runtime()?;
        let default_options = Self::create_default_options(&runtime_info);

        Ok(UniversalFetch {
            runtime_info,
            default_options,
        })
    }

    /// Fetch text data from a URL
    #[wasm_bindgen(js_name = fetchText)]
    pub async fn fetch_text(&self, url: &str) -> Result<String, JsValue> {
        let response = self.fetch_internal(url, "text").await?;
        match response.data {
            FetchData::Text(text) => Ok(text),
            FetchData::Binary(_) => Err(JsValue::from_str("Expected text data, got binary")),
        }
    }

    /// Fetch binary data from a URL
    #[wasm_bindgen(js_name = fetchBinary)]
    pub async fn fetch_binary(&self, url: &str) -> Result<Vec<u8>, JsValue> {
        let response = self.fetch_internal(url, "binary").await?;
        match response.data {
            FetchData::Binary(data) => Ok(data),
            FetchData::Text(_) => Err(JsValue::from_str("Expected binary data, got text")),
        }
    }

    /// Fetch binary data and return as Uint8Array
    #[wasm_bindgen(js_name = fetchBinaryAsArray)]
    pub async fn fetch_binary_as_array(&self, url: &str) -> Result<Uint8Array, JsValue> {
        let data = self.fetch_binary(url).await?;
        Ok(Uint8Array::from(data.as_slice()))
    }

    /// Get runtime information for debugging
    #[wasm_bindgen(js_name = getRuntimeInfo)]
    pub fn get_runtime_info(&self) -> String {
        format!(
            "Runtime: {:?}, HasFetch: {}, HasRequest: {}",
            self.runtime_info.runtime_type,
            self.runtime_info.has_fetch,
            self.runtime_info.has_request_class
        )
    }

    /// Set custom headers for all requests
    #[wasm_bindgen(js_name = setDefaultHeaders)]
    pub fn set_default_headers(&mut self, headers: &Object) -> Result<(), JsValue> {
        let headers_obj = Object::new();

        // Copy existing headers
        if let Ok(existing_headers) =
            Reflect::get(&self.default_options, &JsValue::from_str("headers"))
        {
            if !existing_headers.is_undefined() {
                let existing_obj = existing_headers.dyn_into::<Object>()?;
                let keys = Object::keys(&existing_obj);
                for i in 0..keys.length() {
                    if let Some(key) = keys.get(i).as_string() {
                        let value = Reflect::get(&existing_obj, &JsValue::from_str(&key))?;
                        Reflect::set(&headers_obj, &JsValue::from_str(&key), &value)?;
                    }
                }
            }
        }

        // Add new headers
        let keys = Object::keys(headers);
        for i in 0..keys.length() {
            if let Some(key) = keys.get(i).as_string() {
                let value = Reflect::get(headers, &JsValue::from_str(&key))?;
                Reflect::set(&headers_obj, &JsValue::from_str(&key), &value)?;
            }
        }

        Reflect::set(
            &self.default_options,
            &JsValue::from_str("headers"),
            &headers_obj,
        )?;
        Ok(())
    }

    /// Set timeout for requests (in milliseconds)
    #[wasm_bindgen(js_name = setTimeout)]
    pub fn set_timeout(&mut self, timeout_ms: u32) -> Result<(), JsValue> {
        Reflect::set(
            &self.default_options,
            &JsValue::from_str("timeout"),
            &JsValue::from_f64(timeout_ms as f64),
        )?;
        Ok(())
    }
}

impl UniversalFetch {
    /// Detect the JavaScript runtime environment
    fn detect_runtime() -> Result<RuntimeInfo, JsValue> {
        let global = js_sys::global();

        // Check for specific runtime globals
        let runtime_type = if Self::has_property(&global, "window")? {
            RuntimeType::Browser
        } else if Self::has_property(&global, "process")? {
            // Check if it's Node.js or Bun
            let process = Reflect::get(&global, &JsValue::from_str("process"))?;
            if let Ok(versions) = Reflect::get(&process, &JsValue::from_str("versions")) {
                if Self::has_property(&versions.dyn_into::<Object>()?, "bun")? {
                    RuntimeType::Bun
                } else {
                    RuntimeType::NodeJs
                }
            } else {
                RuntimeType::NodeJs
            }
        } else if Self::has_property(&global, "Deno")? {
            RuntimeType::Deno
        } else {
            RuntimeType::Unknown
        };

        // Check for fetch availability
        let has_fetch = Self::has_property(&global, "fetch")?;

        // Check for Request class availability
        let has_request_class = Self::has_property(&global, "Request")?;

        Ok(RuntimeInfo {
            runtime_type,
            has_fetch,
            has_request_class,
        })
    }

    /// Check if an object has a property
    fn has_property(obj: &Object, prop: &str) -> Result<bool, JsValue> {
        Ok(!Reflect::get(obj, &JsValue::from_str(prop))?.is_undefined())
    }

    /// Create default fetch options based on runtime
    fn create_default_options(runtime_info: &RuntimeInfo) -> Object {
        let options = Object::new();

        // Set method
        Reflect::set(
            &options,
            &JsValue::from_str("method"),
            &JsValue::from_str("GET"),
        )
        .unwrap_or_default();

        // Set headers
        let headers = Object::new();
        Reflect::set(
            &headers,
            &JsValue::from_str("User-Agent"),
            &JsValue::from_str("readap-wasm/1.0.0"),
        )
        .unwrap_or_default();

        // Runtime-specific headers and options
        match runtime_info.runtime_type {
            RuntimeType::Browser => {
                // Browser-specific settings
                Reflect::set(
                    &options,
                    &JsValue::from_str("mode"),
                    &JsValue::from_str("cors"),
                )
                .unwrap_or_default();
                Reflect::set(
                    &options,
                    &JsValue::from_str("credentials"),
                    &JsValue::from_str("omit"),
                )
                .unwrap_or_default();
            }
            RuntimeType::NodeJs => {
                // Node.js specific settings
                Reflect::set(
                    &headers,
                    &JsValue::from_str("Accept"),
                    &JsValue::from_str("*/*"),
                )
                .unwrap_or_default();
            }
            RuntimeType::Bun => {
                // Bun specific settings
                Reflect::set(
                    &headers,
                    &JsValue::from_str("Accept"),
                    &JsValue::from_str("*/*"),
                )
                .unwrap_or_default();
            }
            RuntimeType::Deno => {
                // Deno specific settings
                Reflect::set(
                    &headers,
                    &JsValue::from_str("Accept"),
                    &JsValue::from_str("*/*"),
                )
                .unwrap_or_default();
            }
            RuntimeType::Unknown => {
                // Minimal settings for unknown runtime
            }
        }

        Reflect::set(&options, &JsValue::from_str("headers"), &headers).unwrap_or_default();

        options
    }

    /// Internal fetch implementation using unified approach
    async fn fetch_internal(
        &self,
        url: &str,
        response_type: &str,
    ) -> Result<FetchResponse, JsValue> {
        if self.runtime_info.has_fetch {
            self.fetch_generic(url, response_type).await
        } else {
            Err(JsValue::from_str("No fetch implementation available"))
        }
    }

    /// Universal fetch implementation using js_sys::Reflect for all runtimes
    async fn fetch_generic(
        &self,
        url: &str,
        response_type: &str,
    ) -> Result<FetchResponse, JsValue> {
        let global = js_sys::global();
        let fetch_fn =
            Reflect::get(&global, &JsValue::from_str("fetch"))?.dyn_into::<js_sys::Function>()?;

        // Create request options
        let options = self.create_request_options()?;

        // Make the fetch call
        let promise = fetch_fn
            .call2(&global, &JsValue::from_str(url), &options)?
            .dyn_into::<Promise>()?;

        let response = JsFuture::from(promise).await?;

        // Extract status using Reflect API
        let status = if let Ok(status_val) = Reflect::get(&response, &JsValue::from_str("status")) {
            status_val.as_f64().unwrap_or(0.0) as u16
        } else {
            200 // Assume success if we can't get status
        };

        let status_text = if let Ok(status_text_val) =
            Reflect::get(&response, &JsValue::from_str("statusText"))
        {
            status_text_val.as_string().unwrap_or_default()
        } else {
            "OK".to_string()
        };

        // Check if the request was successful
        let ok = if let Ok(ok_val) = Reflect::get(&response, &JsValue::from_str("ok")) {
            ok_val.as_bool().unwrap_or(status >= 200 && status < 300)
        } else {
            status >= 200 && status < 300
        };

        if !ok {
            return Err(JsValue::from_str(&format!(
                "HTTP Error {}: {}",
                status, status_text
            )));
        }

        // Extract data based on type using Reflect API
        let data = match response_type {
            "text" => {
                let text_method = Reflect::get(&response, &JsValue::from_str("text"))?
                    .dyn_into::<js_sys::Function>()?;
                let text_promise = text_method.call0(&response)?.dyn_into::<Promise>()?;
                let text_result = JsFuture::from(text_promise).await?;
                let text = text_result.as_string().unwrap_or_default();
                FetchData::Text(text)
            }
            "binary" => {
                let array_buffer_method =
                    Reflect::get(&response, &JsValue::from_str("arrayBuffer"))?
                        .dyn_into::<js_sys::Function>()?;
                let array_buffer_promise = array_buffer_method
                    .call0(&response)?
                    .dyn_into::<Promise>()?;
                let array_buffer_result = JsFuture::from(array_buffer_promise).await?;
                let array_buffer = array_buffer_result.dyn_into::<ArrayBuffer>()?;
                let uint8_array = Uint8Array::new(&array_buffer);
                FetchData::Binary(uint8_array.to_vec())
            }
            _ => return Err(JsValue::from_str("Invalid response type")),
        };

        Ok(FetchResponse {
            status,
            status_text,
            data,
            headers: Object::new(),
        })
    }

    /// Create request options object
    fn create_request_options(&self) -> Result<Object, JsValue> {
        let options = Object::new();

        // Copy from default options
        let keys = Object::keys(&self.default_options);
        for i in 0..keys.length() {
            if let Some(key) = keys.get(i).as_string() {
                let value = Reflect::get(&self.default_options, &JsValue::from_str(&key))?;
                Reflect::set(&options, &JsValue::from_str(&key), &value)?;
            }
        }

        Ok(options)
    }
}

/// Helper function to create a global fetch client instance
#[wasm_bindgen(js_name = createUniversalFetch)]
pub fn create_universal_fetch() -> Result<UniversalFetch, JsValue> {
    UniversalFetch::new()
}

/// Standalone text fetch function for convenience
#[wasm_bindgen(js_name = universalFetchText)]
pub async fn universal_fetch_text(url: &str) -> Result<String, JsValue> {
    let fetcher = UniversalFetch::new()?;
    fetcher.fetch_text(url).await
}

/// Standalone binary fetch function for convenience
#[wasm_bindgen(js_name = universalFetchBinary)]
pub async fn universal_fetch_binary(url: &str) -> Result<Vec<u8>, JsValue> {
    let fetcher = UniversalFetch::new()?;
    fetcher.fetch_binary(url).await
}
