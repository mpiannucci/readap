// Example usage of readap-wasm
// This shows how to use the updated WASM bindings with the correct API

import { 
  parse_dds, 
  parse_das, 
  parse_dods,
  JsUrlBuilder,
  create_query_builder,
  get_variable_info,
  get_coordinate_info
} from './pkg/readap_wasm.js';

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
    Grid {
     ARRAY:
        Float32 wind_speed[time = 100][latitude = 5][longitude = 10];
     MAPS:
        Int32 time[time = 100];
        Float32 latitude[latitude = 5];
        Float32 longitude[longitude = 10];
    } wind_speed;
} test_dataset;`;

// Example DAS content
const dasContent = `Attributes {
    time {
        String long_name "Epoch Time";
        String short_name "time";
        String standard_name "time";
        String units "seconds since 1970-01-01 00:00:00 UTC";
    }
    temperature {
        String long_name "Sea Surface Temperature";
        String short_name "sst";
        String standard_name "sea_surface_temperature";
        String units "degrees_C";
        Float32 _FillValue -999.0;
    }
}`;

async function example() {
  try {
    console.log('=== Parsing DDS ===');
    const dataset = parse_dds(ddsContent);
    console.log('Dataset name:', dataset.name);
    console.log('Variables:', dataset.variables);
    console.log('Coordinates:', dataset.coordinates);
    console.log('DDS Values:', dataset.values.length);

    console.log('\n=== Parsing DAS ===');
    const attributes = parse_das(dasContent);
    console.log('Attributes:', Object.keys(attributes));
    console.log('Time attributes:', attributes.time);

    console.log('\n=== Variable Information ===');
    const tempInfo = get_variable_info(ddsContent, 'temperature');
    console.log('Temperature variable info:', tempInfo);

    console.log('\n=== Coordinate Information ===');
    const timeInfo = get_coordinate_info(ddsContent, 'time');
    console.log('Time coordinate info:', timeInfo);

    console.log('\n=== URL Builder ===');
    const baseUrl = 'https://example.com/data/ocean';
    
    // Basic URL generation
    let builder = new JsUrlBuilder(baseUrl);
    console.log('DAS URL:', builder.dasUrl());
    console.log('DDS URL:', builder.ddsUrl());
    console.log('Basic DODS URL:', builder.dodsUrl());

    // Variable selection
    builder = new JsUrlBuilder(baseUrl)
      .addVariable('temperature')
      .addVariable('wind_speed');
    console.log('With variables:', builder.dodsUrl());

    // Single index constraint
    builder = new JsUrlBuilder(baseUrl)
      .addVariable('temperature')
      .addSingleIndex('temperature', 5);
    console.log('With single index:', builder.dodsUrl());

    // Range constraint
    builder = new JsUrlBuilder(baseUrl)
      .addVariable('temperature')
      .addRange('temperature', 0, 10);
    console.log('With range:', builder.dodsUrl());

    // Range with stride
    builder = new JsUrlBuilder(baseUrl)
      .addVariable('temperature')
      .addRangeWithStride('temperature', 0, 20, 2);
    console.log('With stride:', builder.dodsUrl());

    // Multidimensional constraint
    builder = new JsUrlBuilder(baseUrl)
      .addVariable('temperature')
      .addMultidimensionalConstraint('temperature', [
        { start: 0, end: 10 },      // time dimension
        5,                          // latitude dimension (single index)
        { start: 0, end: 8, stride: 2 }  // longitude dimension with stride
      ]);
    console.log('Multidimensional:', builder.dodsUrl());

    console.log('\n=== Query Builder ===');
    const queryBuilder = create_query_builder(ddsContent, baseUrl);
    console.log('Query builder:', queryBuilder);

  } catch (error) {
    console.error('Error:', error);
  }
}

// Run the example
example();