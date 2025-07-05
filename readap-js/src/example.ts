import { DAPClient } from './index.js';

async function example() {
  // Create a client for an OpenDAP dataset
  const client = new DAPClient('https://example.com/opendap/data-source');

  // WASM module initializes automatically

  try {
    // Get dataset information with strongly typed response
    const info = await client.getDatasetInfo();
    console.log('Dataset:', info.name);
    console.log('Variables:', info.variables.map(v => `${v.name} (${v.dataType})`));
    console.log('Coordinates:', info.coordinates.map(c => `${c.name} [${c.size}]`));

    // Show variable details with type safety
    for (const variable of info.variables) {
      console.log(`Variable: ${variable.name}`);
      console.log(`  Type: ${variable.variableType}`);
      console.log(`  Data Type: ${variable.dataType}`);
      console.log(`  Coordinates: ${variable.coordinates.join(', ')}`);
      console.log(`  Dimensions: ${variable.dimensions.map(d => `${d.name}=${d.size}`).join(', ')}`);
    }

    // Fetch data for a specific variable with constraints
    const data = await client.fetchData('temperature', {
      constraints: {
        time: [0, 10],    // Time indices 0-10
        lat: 45,          // Single latitude index
        lon: [120, 130]   // Longitude range
      }
    });

    console.log('Fetched data type:', data.metadata.type);
    console.log('Data shape:', data.data.length);
    console.log('Variable metadata:', data.metadata);
  } catch (error) {
    console.error('Error:', error);
  }
}

// Only run if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  example();
}