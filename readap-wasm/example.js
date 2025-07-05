import init, {
  parse_dds,
  parse_das,
  parse_dods,
  UrlBuilder,
} from './pkg/readap_wasm.js';
import { readFileSync } from 'fs';

// Example DDS content
const ddsContent = `Dataset {
    Float32 latitude[latitude = 5];
    Float32 longitude[longitude = 10];
    Int32 time[time = 100];
    Grid {
     ARRAY:
        Float32 temperature[time = 100][latitude = 5][longitude = 10];
     MAPS:
        Int32 time[time = 100];
        Float32 latitude[latitude = 5];
        Float32 longitude[longitude = 10];
    } temperature;
} test_dataset;`;

// Example DAS content
const dasContent = `Attributes {
    time {
        String long_name "Epoch Time";
        String units "seconds since 1970-01-01 00:00:00 UTC";
    }
    temperature {
        String long_name "Sea Surface Temperature";
        String units "degrees_C";
        Float32 _FillValue -999.0;
    }
}`;

async function run() {
  // Load WASM file for Node.js
  const wasmBytes = readFileSync('./pkg/readap_wasm_bg.wasm');
  await init(wasmBytes);

  console.log('readap-wasm loaded successfully');
  
  try {
    console.log('\n=== Parsing DDS ===');
    const dataset = parse_dds(ddsContent);
    console.log('Dataset name:', dataset.name);
    console.log('Variables:', dataset.variables);
    console.log('Coordinates:', dataset.coordinates);
    console.log('DDS Values:', dataset.values.length);

    console.log('\n=== Parsing DAS ===');
    const attributes = parse_das(dasContent);
    console.log('Attributes:', Object.keys(attributes));
    if (attributes.time) {
      console.log('Time attributes:', attributes.time);
    }

    console.log('\n=== URL Builder ===');
    const baseUrl = 'https://example.com/data/ocean';
    
    // Basic URL generation
    let builder = new UrlBuilder(baseUrl);
    console.log('DAS URL:', builder.dasUrl());
    console.log('DDS URL:', builder.ddsUrl());
    console.log('Basic DODS URL:', builder.dodsUrl());

    // Variable selection
    builder = new UrlBuilder(baseUrl)
      .addVariable('temperature');
    console.log('With variable:', builder.dodsUrl());

    // Range constraint
    builder = new UrlBuilder(baseUrl)
      .addVariable('temperature')
      .addRange('temperature', 0, 10, null);
    console.log('With range:', builder.dodsUrl());

    // Range with stride
    builder = new UrlBuilder(baseUrl)
      .addVariable('temperature')
      .addRange('temperature', 0, 20, 2);
    console.log('With stride:', builder.dodsUrl());

    console.log('\n=== Zero-Copy Data Access Demo ===');
    console.log('Note: parse_dods() would be used with actual binary DODS data');
    console.log('Example usage:');
    console.log('  const dodsData = parse_dods(binaryBytes);');
    console.log('  const variables = dodsData.getVariables();');
    console.log('  const tempData = dodsData.getVariableData("temperature"); // Float32Array');
    console.log('  const timeData = dodsData.getVariableData("time"); // Int32Array');

  } catch (error) {
    console.error('Error:', error);
  }
}

run();