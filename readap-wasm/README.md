# readap-wasm

**WebAssembly bindings for the readap OpenDAP parser library**

A WebAssembly wrapper around the [readap](../readap) Rust library, enabling OpenDAP data parsing in web browsers and Node.js environments.

## About

This package provides WebAssembly bindings for the readap library, allowing you to parse OpenDAP binary data and metadata directly in JavaScript/TypeScript applications. It supports parsing DAS (Data Attribute Structure), DDS (Data Descriptor Structure), and DODS (Data Object) formats.

## Installation

Install from NPM (once published):

```bash
npm install readap-wasm
```

Or use in a web page:

```html
<script type="module">
  import init, { greet } from './pkg/readap_wasm.js';
  
  async function run() {
    await init();
    greet();
  }
  
  run();
</script>
```

## Development

### Build the WebAssembly package

```bash
wasm-pack build
```

### Test in headless browsers

```bash
wasm-pack test --headless --firefox
```

### Publish to NPM

```bash
wasm-pack publish
```

## Usage

### Basic Setup

```typescript
import init, { OpenDAPDataset } from '@readap-wasm/readap-wasm';

async function main() {
  await init();  // Initialize WASM module
  
  // Load dataset with automatic metadata fetching
  const dataset = await OpenDAPDataset.fromURL('http://example.com/ocean.nc');
  
  // Check available variables
  console.log('Variables:', dataset.getVariableNames());
  
  // Get simple variable data
  const tempData = await dataset.getVariable('temperature');
  console.log('Temperature:', tempData.data); // Float64Array or appropriate typed array
}
```

### Advanced Data Selection

```typescript
// Index-based selection (isel) - select by array indices
const indexSelection = dataset.isel({
  time: 0,           // first time step
  lat: [10, 20],     // latitude indices 10-20
  lon: 50            // longitude index 50
});

// Value-based selection (sel) - select by coordinate values with nearest neighbor
await dataset.loadCoordinates('time');  // Load coordinates for value lookup
await dataset.loadCoordinates('lat');
await dataset.loadCoordinates('lon');

const valueSelection = dataset.sel({
  time: "2023-01-15T12:00:00Z",  // nearest time
  lat: [40.0, 45.0],             // latitude range 40-45°N
  lon: -70.0                     // nearest to -70°W
});

// Get data with selection applied
const selectedData = await dataset.getVariable('temperature', valueSelection);

// Chain selections for complex queries
const surface = dataset
  .sel({ depth: 0 })                    // surface level
  .isel({ time: [0, 1, 2] })           // first 3 time steps
  .sel({ lat: [35, 45], lon: [-80, -60] }); // geographic subset
```

### Multiple Variables and Batch Operations

```typescript
// Get multiple variables efficiently
const oceanData = await dataset.getVariables(['temperature', 'salinity', 'velocity']);

// Access individual variable data
console.log('Temperature data:', oceanData.temperature.data);
console.log('Salinity data:', oceanData.salinity.data);
```

### Low-Level API Usage

```typescript
// Manual URL building
const urlBuilder = new OpenDAPUrlBuilder("http://example.com/data");
console.log(urlBuilder.dasUrl());  // http://example.com/data.das
console.log(urlBuilder.ddsUrl());  // http://example.com/data.dds

// Direct constraint building
const constraints = new ConstraintBuilder()
  .isel({ time: { type: "single", value: 0 } })
  .sel({ lat: { type: "range", min: 40.0, max: 50.0 } });

const dodsUrl = urlBuilder.dodsUrl(constraints.build());

// Parse formats directly
const dasResult = OpenDAPDataset.fromDAS(dasText);
const ddsResult = OpenDAPDataset.fromDDS(ddsText);
```

## Features

### 🚀 **High-Level API**
* **Automatic Data Fetching**: Seamless HTTP requests with built-in error handling
* **xarray-style Selection**: Intuitive `isel` (index) and `sel` (value) selection patterns
* **Nearest Neighbor Lookup**: Automatic coordinate value → index mapping with binary search
* **Lazy Loading**: Coordinates and metadata loaded only when needed for optimal performance
* **Typed Arrays**: Efficient JavaScript typed arrays (Float64Array, Int32Array, etc.) for zero-copy data transfer

### 🔧 **Low-Level Control**
* **URL Builder System**: Programmatically construct OpenDAP URLs with constraints
* **Direct Format Parsing**: Parse DAS, DDS, and DODS formats independently
* **Constraint Building**: Flexible constraint syntax for complex data selections
* **Network-Free Core**: Underlying readap library works without network dependencies

### 📊 **Data Selection Capabilities**
* **Index-based Selection (isel)**: Select data by array indices with ranges and multiple values
* **Value-based Selection (sel)**: Select data by coordinate values with nearest neighbor matching
* **Chained Selections**: Combine multiple selection operations for complex queries
* **Range Support**: Efficient range selections for time series and spatial data
* **Gridded Coordinates**: Intuitive handling of multi-dimensional coordinate systems

### 🌐 **Web Compatibility**
* **Full WebAssembly compatibility** for browser and Node.js environments
* **CORS Support**: Proper handling of cross-origin requests
* **Built with [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen)** for seamless JavaScript integration
* **Error Handling**: Comprehensive HTTP and parsing error handling with meaningful messages
* **Includes [`console_error_panic_hook`](https://github.com/rustwasm/console_error_panic_hook)** for better debugging

### ⚡ **Performance Features**
* **Zero-copy data transfer** between WASM and JavaScript where possible
* **Efficient binary search** algorithms for coordinate lookup
* **Minimal network requests** through intelligent constraint building
* **Smart coordinate caching** to avoid redundant data fetching

## License

[MIT](LICENSE)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
