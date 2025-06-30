# ReadAP WASM Library - API Summary

A powerful TypeScript/WebAssembly library for OpenDAP data access with automatic fetching, xarray-style selection, and efficient typed array handling.

## Features

### ✅ **Complete Implementation**

- **Automatic Data Fetching**: Uses JavaScript fetch API for seamless data access
- **Network-free Core**: The underlying readap library remains portable across platforms
- **xarray-style Selection**: Intuitive `isel` (index) and `sel` (value) selection patterns
- **Nearest Neighbor Lookup**: Automatic coordinate value → index mapping
- **Typed Arrays**: Efficient JavaScript typed arrays (Float64Array, Int32Array, etc.)
- **Lazy Loading**: Coordinates and data loaded only when needed
- **Error Handling**: Comprehensive HTTP and parsing error handling

## High-Level API

### Dataset Creation

```typescript
// Automatic metadata loading
const dataset = await OpenDAPDataset.fromURL("http://example.com/data.nc");

// Manual control (lazy loading)
const dataset = OpenDAPDataset.fromURLLazy("http://example.com/data.nc");
await dataset.parseDAS(await fetch(dataset.dasUrl()).then(r => r.text()));
await dataset.parseDDS(await fetch(dataset.ddsUrl()).then(r => r.text()));
```

### Data Access

```typescript
// Simple variable access
const tempData = await dataset.getVariable("temperature");
console.log(tempData.data); // Float64Array or appropriate typed array

// Multiple variables
const data = await dataset.getVariables(["temperature", "pressure"]);

// With constraints
const selection = dataset.isel({ time: 0, lat: [10, 20] });
const slicedData = await dataset.getVariable("temperature", selection);
```

### Selection API

```typescript
// Index-based selection (isel)
const indexSel = dataset.isel({
  time: { type: "single", value: 0 },
  lat: { type: "range", start: 10, end: 20 },
  lon: { type: "multiple", values: [0, 5, 10] }
});

// Value-based selection (sel) with nearest neighbor
const valueSel = dataset.sel({
  time: "2023-01-15",     // nearest neighbor to time coordinate
  lat: [40.0, 50.0],      // range between coordinate values
  lon: -74.0              // single coordinate value (nearest neighbor)
});

// Chained selections
const combined = dataset
  .isel({ time: 0 })
  .sel({ lat: [40, 50] });
```

### Coordinate Management

```typescript
// Automatic coordinate loading for sel operations
await dataset.loadCoordinates("time");
await dataset.loadCoordinates("lat");

// Manual coordinate addition
const timeCoords = new Float64Array([0, 6, 12, 18, 24]); // hours
dataset.addCoordinates("time", timeCoords);
```

## Low-Level API

### URL Building

```typescript
const urlBuilder = new OpenDAPUrlBuilder("http://example.com/data");
console.log(urlBuilder.dasUrl());  // http://example.com/data.das
console.log(urlBuilder.ddsUrl());  // http://example.com/data.dds
console.log(urlBuilder.dodsUrl("temperature[0:10]")); // with constraints
```

### Constraint Building

```typescript
const builder = new ConstraintBuilder()
  .isel({ time: { type: "single", value: 0 } })
  .sel({ lat: { type: "range", min: 40.0, max: 50.0 } });

console.log(builder.build()); // "time[0],lat[nearest_indices]"
```

### Direct Parsing

```typescript
// Parse OpenDAP formats directly
const dasResult = OpenDAPDataset.fromDAS(dasText);
const ddsResult = OpenDAPDataset.fromDDS(ddsText);
const dodsResult = dataset.parseDODS(dodsBytes); // Uint8Array → Object with typed arrays
```

## Example Usage

```typescript
import init, { OpenDAPDataset } from '@readap-wasm/readap-wasm';

async function main() {
  await init();
  
  // Load dataset with automatic metadata fetching
  const dataset = await OpenDAPDataset.fromURL('http://example.com/ocean.nc');
  
  // Check available variables
  console.log('Variables:', dataset.getVariableNames());
  
  // Load coordinates for value-based selection
  await dataset.loadCoordinates('time');
  await dataset.loadCoordinates('lat');
  await dataset.loadCoordinates('lon');
  
  // Select data using coordinate values (nearest neighbor)
  const selection = dataset.sel({
    time: "2023-01-15T12:00:00Z",
    lat: [40.0, 45.0],  // latitude range
    lon: -70.0          // single longitude
  });
  
  // Fetch temperature data with selection
  const tempData = await dataset.getVariable('temperature', selection);
  console.log('Temperature:', tempData.data); // Float32Array or Float64Array
  
  // Get multiple variables efficiently
  const oceanData = await dataset.getVariables(['temperature', 'salinity', 'velocity']);
  
  // Chain selections for complex queries
  const surface = dataset
    .sel({ depth: 0 })                    // surface level
    .isel({ time: [0, 1, 2] })           // first 3 time steps
    .sel({ lat: [35, 45], lon: [-80, -60] }); // geographic subset
    
  const surfaceTemp = await dataset.getVariable('temperature', surface);
}
```

## Architecture Benefits

1. **Portable Core**: The readap library has no network dependencies, making it usable in any Rust environment
2. **Efficient WASM**: Direct memory transfer between Rust and JavaScript using typed arrays
3. **Smart Caching**: Coordinates are cached to avoid redundant network requests
4. **Error Resilience**: Comprehensive error handling for network failures and data parsing issues
5. **TypeScript Ready**: Full type safety with appropriate typed array selection based on data types

## Performance Features

- **Zero-copy data transfer** where possible between WASM and JavaScript
- **Lazy coordinate loading** - coordinates fetched only when needed for `sel` operations
- **Efficient nearest neighbor** lookup using binary search algorithms
- **Minimal network requests** through intelligent constraint building
- **Browser optimization** with proper fetch API usage and CORS support

This implementation provides both the low-level control needed for advanced users and the high-level convenience required for typical scientific data workflows.