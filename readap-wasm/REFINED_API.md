# Refined WASM Bindings for readap

## Overview
The WASM bindings have been refined to closely match the actual readap Rust API, providing a JavaScript-friendly interface for parsing OpenDAP datasets.

## API Reference

### Core Parsing Functions

#### `parse_dds(content: string): object`
Parses DDS (Dataset Descriptor Structure) content and returns a JavaScript object with:
- `name`: Dataset name
- `values`: Array of DDS values (Arrays, Grids, Structures, Sequences)
- `variables`: Array of variable names
- `coordinates`: Array of coordinate names

#### `parse_das(content: string): object`
Parses DAS (Dataset Attribute Structure) content and returns a nested object structure:
```javascript
{
  "variable_name": {
    "attribute_name": {
      "name": "attribute_name",
      "dataType": "String|Int32|Float32|...",
      "value": actual_value
    }
  }
}
```

#### `parse_dods(bytes: Uint8Array): object`
Parses DODS (Dataset Data Structure) binary content and returns:
- `dataset`: DDS dataset information
- `variables`: Array of variable names

### URL Builder

#### `JsUrlBuilder`
JavaScript wrapper for the Rust UrlBuilder with fluent API:

```javascript
const builder = new JsUrlBuilder("https://example.com/data.nc");

// Basic URLs
builder.dasUrl()  // "https://example.com/data.nc.das"
builder.ddsUrl()  // "https://example.com/data.nc.dds"
builder.dodsUrl() // "https://example.com/data.nc.dods"

// Variable selection
builder.addVariable("temperature")
       .addVariables(["pressure", "humidity"])

// Constraints
builder.addSingleIndex("time", 5)
       .addRange("latitude", 0, 10, null) // no stride
       .addRange("longitude", -180, 180, 2) // with stride

// Multidimensional constraints
builder.addMultidimensionalConstraint("temperature", [
  5,                              // Single index
  { start: 0, end: 10 },         // Range without stride
  { start: 0, end: 20, stride: 2 } // Range with stride
])

// Utilities
builder.clearVariables()
       .clearConstraints()
       .clearAll()
       .clone()
```

## Type Mappings

### DataType Enum
- `Byte`, `Int16`, `UInt16`, `Int32`, `UInt32`
- `Float32`, `Float64`
- `String`, `URL`

### DDS Value Types
- `Array`: Multi-dimensional array with coordinates
- `Grid`: Array with associated coordinate maps
- `Structure`: Nested structure with fields
- `Sequence`: Variable-length sequence with fields

## Universal Package Support

The package builds with universal support for:
- **Node.js**: CommonJS and ES modules
- **Browsers**: Direct import with bundlers
- **Deno**: Native ES module support
- **Bun**: Native support

### Usage Examples

```javascript
// Node.js (CommonJS)
const loadWasm = require('readap-wasm/universal');
const wasm = await loadWasm();
const result = wasm.parse_dds(ddsContent);

// Node.js/Deno/Bun (ES Modules)
import loadWasm from 'readap-wasm/universal';
const wasm = await loadWasm();

// Browser (with bundler)
import * as wasm from 'readap-wasm';
await wasm.default();
const builder = new wasm.JsUrlBuilder("https://example.com/data");
```

## Key Improvements

1. **API Alignment**: Matches the actual readap Rust API exactly
2. **Rich Type Support**: Full support for all OpenDAP data types
3. **Builder Pattern**: Fluent URL building with method chaining
4. **Universal Package**: Single build works across all JavaScript runtimes
5. **Error Handling**: Proper JavaScript-friendly error messages
6. **Type Safety**: Comprehensive type conversions and validations

## Build Process

```bash
# Build for all targets
./build-universal.sh

# The package is created in pkg/ with universal runtime support
```