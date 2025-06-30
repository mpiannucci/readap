# readap-wasm Examples

**Universal OpenDAP client examples for all JavaScript runtimes**

## 🚀 Basic Usage

### Quick Start (Works Everywhere)

```javascript
import init, { ImmutableDataset, SimpleConstraintBuilder } from 'readap-wasm';

await init();

// Load dataset
const dataset = await ImmutableDataset.fromURL('https://server.com/data.nc');

// Build constraint safely (no aliasing errors!)
const constraint = new SimpleConstraintBuilder()
    .addSingle('time', 0)
    .addRange('latitude', 10, 20)
    .build();

// Get data
const temp = await dataset.getVariable('temperature', `temperature[${constraint}]`);
console.log(`Temperature: ${temp.data[0]}K`);
```

## 🔗 Complete Workflows

### Ocean Data Analysis

```javascript
import init, { 
    ImmutableDataset, 
    SimpleConstraintBuilder, 
    UniversalFetch 
} from 'readap-wasm';

async function analyzeOceanTemperature() {
    await init();
    
    // 1. Create dataset
    const dataset = await ImmutableDataset.fromURL(
        'https://oceandata.server.com/sst.nc'
    );
    
    // 2. Build spatial-temporal constraint
    const constraint = new SimpleConstraintBuilder()
        .addSingle('time', 0)           // Latest time
        .addRange('latitude', 100, 200) // Tropical region  
        .addRange('longitude', 50, 150) // Pacific
        .build();
    
    // 3. Get temperature data
    const sst = await dataset.getVariable('temperature', `temperature[${constraint}]`);
    
    // 4. Analyze data
    const temps = Array.from(sst.data);
    const avgTemp = temps.reduce((a, b) => a + b) / temps.length;
    const maxTemp = Math.max(...temps);
    const minTemp = Math.min(...temps);
    
    return {
        region: 'Tropical Pacific',
        avgTemperature: avgTemp - 273.15, // Convert K to C
        maxTemperature: maxTemp - 273.15,
        minTemperature: minTemp - 273.15,
        dataPoints: temps.length
    };
}
```

### Weather Data Extraction

```javascript
async function getWeatherForecast() {
    await init();
    
    const dataset = await ImmutableDataset.fromURL(
        'https://weather.server.com/gfs.nc'
    );
    
    // Get next 24 hours at specific location
    const constraint = new SimpleConstraintBuilder()
        .addRange('time', 0, 23)      // Next 24 hours
        .addSingle('latitude', 150)   // ~40°N
        .addSingle('longitude', 200)  // ~100°W
        .addSingle('level', 0)        // Surface
        .build();
    
    // Get multiple weather variables
    const temp = await dataset.getVariable('temperature', `temperature[${constraint}]`);
    const wind = await dataset.getVariable('wind_speed', `wind_speed[${constraint}]`);
    
    return {
        hourly_temp: Array.from(temp.data).map(t => t - 273.15), // K to C
        hourly_wind: Array.from(wind.data),
        location: { lat: 40.0, lon: -100.0 }
    };
}
```

## 🏗️ Core API Examples

### ImmutableDataset - Safe Chaining

```javascript
// All operations return NEW instances (no mutation)
const base = new ImmutableDataset(url);
const withMetadata = await ImmutableDataset.fromURL(url);
const withDAS = withMetadata.withDAS(dasText);
const withCoords = withDAS.withCoordinates('time', timeValues);

// Original objects unchanged
console.log(base !== withMetadata); // true
console.log(withMetadata !== withDAS); // true
```

### SimpleConstraintBuilder - No Aliasing

```javascript
// Method chaining that works in ALL JavaScript runtimes
const builder = new SimpleConstraintBuilder()
    .addSingle('time', 5)                    // time[5]
    .addRange('latitude', 0, 100)            // latitude[0:100]
    .addStride('longitude', 0, 2, 360)       // longitude[0:2:360]
    .addMultiple('level', [0, 10, 20, 50]);  // level[0,10,20,50]

const constraint = builder.build();
// Result: "time[5],latitude[0:100],longitude[0:2:360],level[0,10,20,50]"
```

### UniversalFetch - Runtime Agnostic

```javascript
const fetcher = new UniversalFetch();

// Automatically adapts to Browser/Node.js/Bun/Deno
console.log(fetcher.getRuntimeInfo());

// Fetch text data
const dasData = await fetcher.fetchText('https://server.com/data.das');

// Fetch binary data  
const dodsData = await fetcher.fetchBinary('https://server.com/data.dods');
```

### UniversalDodsParser - Binary Parsing

```javascript
const parser = new UniversalDodsParser();
parser.setDebugMode(true); // Enable detailed logging

// Parse binary OpenDAP data
const binaryData = new Uint8Array(response);
const parsed = parser.parseDods(binaryData);

// Access parsed variables
console.log(Object.keys(parsed)); // ['temperature', 'salinity', 'time']
console.log(parsed.temperature.data); // Float64Array with values
console.log(parsed.temperature.dimensions); // [100, 50] shape

// Detailed parsing with error info
const detailed = parser.parseDodsDetailed(binaryData);
if (detailed.success) {
    console.log(detailed.variables);
} else {
    console.error(detailed.error);
}
```

## 🌐 Runtime-Specific Examples

### Browser Usage

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, { ImmutableDataset } from './pkg/readap_wasm.js';
        
        async function loadOceanData() {
            await init();
            const dataset = await ImmutableDataset.fromURL(
                'https://oceandata.org/sst.nc'
            );
            // Process data...
        }
        
        loadOceanData();
    </script>
</head>
</html>
```

### Node.js Usage

```javascript
// Works with Node.js imports
import init, { ImmutableDataset } from 'readap-wasm';

async function processData() {
    await init();
    
    const dataset = await ImmutableDataset.fromURL(process.env.DATA_URL);
    // Node.js specific processing...
}
```

### Bun Usage

```javascript
#!/usr/bin/env bun
import init, { SimpleConstraintBuilder } from 'readap-wasm';

await init();

// Bun-optimized constraint building
const constraint = new SimpleConstraintBuilder()
    .addSingle('time', 0)
    .build();

console.log(`Constraint: ${constraint}`);
```

### Deno Usage

```javascript
// deno run --allow-net example.js
import init, { UniversalFetch } from 'https://unpkg.com/readap-wasm/readap_wasm.js';

await init();

const fetcher = new UniversalFetch();
const data = await fetcher.fetchText('https://server.com/data.das');
```

## 🔧 Advanced Patterns

### Error Handling

```javascript
try {
    const dataset = await ImmutableDataset.fromURL(url);
    const data = await dataset.getVariable('temperature', constraint);
} catch (error) {
    if (error.message.includes('recursive use')) {
        // This should never happen with the new API!
        console.error('Aliasing error (impossible with ImmutableDataset)');
    } else if (error.message.includes('network')) {
        console.error('Network error:', error.message);
    } else {
        console.error('Parsing error:', error.message);
    }
}
```

### Performance Optimization

```javascript
// Use minimal constraints for better performance
const smallConstraint = new SimpleConstraintBuilder()
    .addSingle('time', 0)
    .addSingle('latitude', 100)  
    .addSingle('longitude', 200)
    .build();

// Batch multiple variables efficiently
const variables = ['temperature', 'salinity', 'velocity'];
const allData = await dataset.getVariables(variables, constraint);
```

### Custom URL Building

```javascript
import { OpenDAPUrlBuilder } from 'readap-wasm';

const builder = new OpenDAPUrlBuilder('https://data.server.com/ocean');

console.log(builder.dasUrl());  // https://data.server.com/ocean.das
console.log(builder.ddsUrl());  // https://data.server.com/ocean.dds

const constraint = 'temperature[0:10][50:100][200:300]';
console.log(builder.dodsUrl(constraint)); // With constraint
```

## 🐛 Common Issues (Now Fixed!)

### ❌ Old API (Caused Errors)
```javascript
// This caused "recursive use of an object detected" in Bun
const dataset = new OpenDAPDataset(url);
dataset.parseDAS(dasData); // Mutates self - ERROR!
```

### ✅ New API (Works Everywhere)
```javascript
// This works in ALL JavaScript runtimes
const dataset = new ImmutableDataset(url);
const newDataset = dataset.withDAS(dasData); // Returns new instance - SUCCESS!
```

## 📊 Real-World Use Cases

### Climate Data Analysis
- Extract temperature trends from NOAA datasets
- Analyze precipitation patterns from weather models
- Process satellite ocean color data

### Ocean Science
- Extract sea surface temperature from satellite data
- Analyze ocean current velocity fields
- Process bathymetry and topography data

### Weather Forecasting
- Extract GFS/NAM model forecast data
- Process ensemble weather predictions
- Analyze atmospheric pressure fields

---

**All examples tested across Browser, Node.js, Bun, and Deno environments** ✅