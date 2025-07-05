import init, {
  parse_dds,
  parse_das,
  parse_dods,
  UrlBuilder,
} from './pkg/readap_wasm.js';
import { readFileSync } from 'fs';

// Example DDS content (from real OpenDAP data)
const ddsContent = `Dataset {
    Grid {
     ARRAY:
        Int32 mean_wave_dir[time = 7][frequency = 64][latitude = 1][longitude = 1];
     MAPS:
        Int32 time[time = 7];
        Float32 frequency[frequency = 64];
        Float32 latitude[latitude = 1];
        Float32 longitude[longitude = 1];
    } mean_wave_dir;
} data/swden/44097/44097w9999.nc;`;

// Example DAS content (from real OpenDAP data)
const dasContent = `Attributes {
    time {
        String long_name "Epoch Time";
        String short_name "time";
        String standard_name "time";
        String units "seconds since 1970-01-01 00:00:00 UTC";
    }
    frequency {
        String long_name "Frequency";
        String short_name "frequency";
        String standard_name "frequency";
        String units "Hz";
    }
    mean_wave_dir {
        String long_name "Mean Wave Direction";
        String short_name "alpha1";
        String standard_name "mean_wave_direction";
        String units "degrees_true";
        Int32 _FillValue 999;
    }
    NC_GLOBAL {
        String institution "NOAA National Data Buoy Center";
        String station "44097";
        String comment "Block Island, RI  (154)";
        String location "40.969 N 71.127 W ";
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
    
    // Show details of the Grid variable
    const gridVar = dataset.values.find(v => v.name === 'mean_wave_dir');
    if (gridVar) {
      console.log('Grid variable details:');
      console.log('  Name:', gridVar.name);
      console.log('  Type:', gridVar.dataType);
      console.log('  Dimensions:', gridVar.dimensions);
    }

    console.log('\n=== Parsing DAS ===');
    const attributes = parse_das(dasContent);
    console.log('Attributes:', Object.keys(attributes));
    if (attributes.time) {
      console.log('Time attributes:', attributes.time);
    }
    if (attributes.mean_wave_dir) {
      console.log('Mean wave direction attributes:', attributes.mean_wave_dir);
    }
    if (attributes.NC_GLOBAL) {
      console.log('Global attributes:', attributes.NC_GLOBAL);
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
      .addVariable('mean_wave_dir');
    console.log('With variable:', builder.dodsUrl());

    // Range constraint
    builder = new UrlBuilder(baseUrl)
      .addVariable('mean_wave_dir')
      .addRange('mean_wave_dir', 0, 3, null);
    console.log('With range:', builder.dodsUrl());

    // Range with stride
    builder = new UrlBuilder(baseUrl)
      .addVariable('mean_wave_dir')
      .addRange('mean_wave_dir', 0, 6, 2);
    console.log('With stride:', builder.dodsUrl());

    console.log('\n=== Zero-Copy Data Access Demo ===');
    console.log('Note: parse_dods() would be used with actual binary DODS data');
    console.log('Example usage:');
    console.log('  const dodsData = parse_dods(binaryBytes);');
    console.log('  const variables = dodsData.getVariables();');
    console.log('  const waveData = dodsData.getVariableData("mean_wave_dir"); // Int32Array');
    console.log('  const timeData = dodsData.getVariableData("time"); // Int32Array');

  } catch (error) {
    console.error('Error:', error);
  }
}

run();