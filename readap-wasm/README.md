# readap-wasm

WebAssembly bindings for the readap OpenDAP parser library with zero-copy data access.

## Usage

```bash
# Build
wasm-pack build --target web

# Test
wasm-pack test --headless --firefox
```

## API

### Parsing Functions
- `parse_dds(content)` - Parse DDS response, returns metadata object
- `parse_das(content)` - Parse DAS response, returns attributes object  
- `parse_dods(bytes)` - Parse DODS binary data, returns `DodsData` instance

### DodsData Class
Efficient access to OpenDAP binary data with zero-copy TypedArrays:

- `getVariables()` - Get list of available variables
- `getVariableData(name)` - Get variable data as native TypedArray
- `getVariableInfo(name)` - Get metadata for specific variable
- `getDatasetInfo()` - Get complete dataset metadata

### UrlBuilder Class
Build OpenDAP URLs with constraints:

- `new UrlBuilder(baseUrl)` - Create URL builder
- `addVariable(name)` - Add variable to query
- `addVariableSlice(name, start, end)` - Add variable with slice constraint
- `dodsUrl()` - Generate DODS URL

## Example

```javascript
import init, { parse_dods, UrlBuilder } from './pkg/readap_wasm.js';

await init();

// Parse DODS data
const dodsData = parse_dods(binaryBytes);

// Get variables
const variables = dodsData.getVariables();
console.log('Available variables:', variables);

// Get data as TypedArrays (zero-copy)
const tempData = dodsData.getVariableData('temperature'); // Float32Array
const timeData = dodsData.getVariableData('time'); // Int32Array

// Get metadata
const tempInfo = dodsData.getVariableInfo('temperature');
const datasetInfo = dodsData.getDatasetInfo();

// Build URLs
const builder = new UrlBuilder('https://example.com/data.nc');
const url = builder.addVariable('temperature').dodsUrl();
```

## Data Types

Variable data is returned as appropriate JavaScript TypedArrays:
- `Int8Array` - Byte data
- `Int16Array` - Int16 data
- `Uint16Array` - UInt16 data  
- `Int32Array` - Int32 data
- `Uint32Array` - UInt32 data
- `Float32Array` - Float32 data
- `Float64Array` - Float64 data
- `Array<string>` - String/URL data

## License

MIT