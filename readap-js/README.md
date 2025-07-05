# readap-js

High-level JavaScript/TypeScript client for OpenDAP data access with full type safety.

## Installation

```bash
npm install readap-js
```

## Features

- 🔒 **Fully typed** - Complete TypeScript support with strongly typed interfaces
- 🚀 **High performance** - Uses WebAssembly for fast OpenDAP parsing
- 🌐 **Universal** - Works in Node.js and browsers
- 📊 **Rich metadata** - Access to complete DDS and DAS information
- 🎯 **Constraint support** - Subset data using coordinate constraints

## Usage

```typescript
import { DAPClient } from 'readap-js';

// Create a client
const client = new DAPClient('https://example.com/opendap/dataset');

// Get dataset information with full type safety
const info = await client.getDatasetInfo();
console.log('Dataset:', info.name);
console.log('Variables:', info.variables.map(v => `${v.name} (${v.dataType})`));

// Access strongly typed variable metadata
for (const variable of info.variables) {
  console.log(`${variable.name}: ${variable.variableType}`);
  console.log(`  Data Type: ${variable.dataType}`);
  console.log(`  Coordinates: ${variable.coordinates.join(', ')}`);
  console.log(`  Dimensions: ${variable.dimensions.map(d => `${d.name}=${d.size}`).join(', ')}`);
}

// Fetch data with constraints - returns typed arrays
const data = await client.fetchData('temperature', {
  constraints: {
    time: [0, 10],    // Time indices 0-10
    lat: 45,          // Single latitude index  
    lon: [120, 130]   // Longitude range
  }
});

// data.data is properly typed as Int8Array | Float32Array | etc.
console.log('Data type:', data.metadata.type);
console.log('Data shape:', data.data.length);
```

## API

### DAPClient

#### Constructor

```typescript
new DAPClient(baseUrl: string, options?: DAPClientOptions)
```

**Options:**
- `timeout?: number` - Request timeout in milliseconds (default: 30000)
- `headers?: Record<string, string>` - Additional HTTP headers

#### Methods

- `getDatasetInfo(): Promise<DatasetInfo>` - Get complete dataset metadata
- `fetchData(variableName: string, options?: FetchDataOptions): Promise<VariableData>` - Fetch variable data

### Types

All data structures are fully typed:

```typescript
interface DatasetInfo {
  name: string;
  variables: VariableInfo[];
  coordinates: CoordinateInfo[];
  attributes: DasAttributes;
  dds: DdsDataset;
}

interface VariableInfo {
  name: string;
  dataType: DataType;
  variableType: VariableType;
  coordinates: string[];
  dimensions: Dimension[];
}

interface VariableData {
  name: string;
  data: Int8Array | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array | string[];
  attributes?: Record<string, any>;
  metadata: DdsValue;
}
```

## Development

This package is part of the readap workspace. To build:

```bash
npm run build
```

## License

MIT