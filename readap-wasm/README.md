# readap-wasm

**Universal OpenDAP client for Browser, Node.js, Bun, Deno, and any JavaScript runtime**

A WebAssembly-powered OpenDAP parser with xarray-style data selection. Works everywhere JavaScript runs.

## ✨ Universal Compatibility

```javascript
// Works in ALL JavaScript environments:
// ✅ Browser  ✅ Node.js  ✅ Bun  ✅ Deno
import { ImmutableDataset, SimpleConstraintBuilder } from 'readap-wasm';
```

## 🚀 Quick Start

```javascript
import init, { ImmutableDataset, SimpleConstraintBuilder } from 'readap-wasm';

await init(); // Initialize WebAssembly

// Load dataset with automatic metadata
const dataset = await ImmutableDataset.fromURL('https://server.com/data.nc');

// Build constraints with method chaining (no aliasing errors!)
const constraint = new SimpleConstraintBuilder()
    .addSingle('time', 0)           // time[0]
    .addRange('latitude', 10, 20)   // latitude[10:20]
    .addSingle('longitude', 50)     // longitude[50]
    .build();

// Get temperature data
const temp = await dataset.getVariable('temperature', `temperature[${constraint}]`);
console.log(`Temperature: ${temp.data[0]}K`);
```

## 📦 Installation

```bash
npm install readap-wasm
```

## 🏗️ Core APIs

### ImmutableDataset - Safe Method Chaining

All operations return new instances (no mutation = no runtime errors):

```javascript
const dataset = new ImmutableDataset(url);

// Load metadata immutably
const withMetadata = await dataset.fromURL(url);

// Chain operations safely
const withDAS = dataset.withDAS(dasText);
const withCoords = withDAS.withCoordinates('time', timeValues);

// Original dataset unchanged, new instances created
console.log(dataset !== withDAS); // true
```

### SimpleConstraintBuilder - No Aliasing Errors

Method chaining that works in all runtimes:

```javascript
const builder = new SimpleConstraintBuilder()
    .addSingle('time', 0)
    .addRange('lat', 0, 10)
    .addStride('lon', 0, 2, 20)    // lon[0:2:20]
    .addMultiple('level', [0,5,10]); // level[0,5,10]

const constraint = builder.build(); // "time[0],lat[0:10],lon[0:2:20],level[0,5,10]"
```

### UniversalFetch - Runtime Agnostic

Automatically adapts to your JavaScript environment:

```javascript
const fetcher = new UniversalFetch();
console.log(fetcher.getRuntimeInfo()); // "Runtime: Bun, HasFetch: true"

const data = await fetcher.fetchBinary(url);  // Works everywhere
const text = await fetcher.fetchText(url);   // Adapts to runtime
```

### UniversalDodsParser - Consistent Binary Parsing

Parse OpenDAP binary data reliably across all runtimes:

```javascript
const parser = new UniversalDodsParser();
const binaryData = new Uint8Array(response);

const parsed = parser.parseDods(binaryData);
console.log(parsed.temperature.data); // Float64Array with values
```

## 🔗 Complete Workflow

```javascript
import init, { 
    ImmutableDataset, 
    SimpleConstraintBuilder,
    UniversalFetch, 
    UniversalDodsParser 
} from 'readap-wasm';

async function getOceanData() {
    await init();
    
    // 1. Create immutable dataset
    const dataset = await ImmutableDataset.fromURL('https://ocean.server.com/data');
    
    // 2. Build constraints safely
    const constraint = new SimpleConstraintBuilder()
        .addSingle('time', 0)
        .addRange('latitude', 100, 200)
        .addRange('longitude', 50, 150)
        .build();
    
    // 3. Fetch and parse data
    const temp = await dataset.getVariable('temperature', `temperature[${constraint}]`);
    
    return {
        temperature: Array.from(temp.data),
        dimensions: temp.dimensions,
        units: temp.attributes?.units
    };
}
```

## 🌐 Runtime Support

| Runtime | Status | Notes |
|---------|--------|-------|
| **Browser** | ✅ | Native WebAssembly + Fetch API |
| **Node.js** | ✅ | Automatic polyfill detection |
| **Bun** | ✅ | Optimized for Bun's runtime |
| **Deno** | ✅ | Web standards compliance |
| **Future runtimes** | ✅ | Universal detection & adaptation |

## 🏛️ Architecture

This package was completely refactored for universal compatibility:

- **Phase 1**: Eliminated mutable self references (no more aliasing errors)
- **Phase 2**: Universal runtime infrastructure (fetch + binary parsing)  
- **Phase 3**: Immutable functional API design
- **Phase 4**: Comprehensive cross-runtime testing

### Key Improvements

- ✅ **No mutable self patterns** - works in all JS engines
- ✅ **Immutable method chaining** - safe state management
- ✅ **Runtime-agnostic networking** - adapts automatically
- ✅ **Universal binary parsing** - consistent across platforms
- ✅ **Functional programming patterns** - predictable behavior

## 🔧 Development

```bash
# Build WebAssembly package
wasm-pack build --target web

# Test across runtimes
bun test-universal-compatibility.js
node test-universal-compatibility.js
```

## 📚 Advanced Usage

### String-based Constraints

```javascript
const builder = new StringConstraintBuilder()
    .addConstraint('time[0]')
    .addConstraint('lat[10:20]')
    .addVariable('temperature');
    
const query = builder.build(); // "time[0],lat[10:20],temperature"
```

### Manual DODS Parsing

```javascript
const fetcher = new UniversalFetch();
const parser = new UniversalDodsParser();

const binaryData = await fetcher.fetchBinary(dodsUrl);
const result = parser.parseDods(new Uint8Array(binaryData));

// Access parsed variables
console.log(Object.keys(result)); // ['temperature', 'salinity', 'time']
console.log(result.temperature.data); // Float64Array
```

### URL Building

```javascript
const builder = new OpenDAPUrlBuilder('https://server.com/data');

console.log(builder.dasUrl());  // https://server.com/data.das
console.log(builder.ddsUrl());  // https://server.com/data.dds
console.log(builder.dodsUrl('temp[0][0:10][0:10]')); // with constraints
```

## 🐛 Troubleshooting

**"recursive use of an object detected"** → Fixed! Use `SimpleConstraintBuilder` or `ImmutableDataset`

**Runtime compatibility issues** → The package automatically detects and adapts to your environment

**Binary parsing errors** → `UniversalDodsParser` handles endianness and format differences

## 📄 License

[MIT](LICENSE)

---

**Built with ❤️ for universal JavaScript compatibility**