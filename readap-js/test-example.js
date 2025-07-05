import { DAPClient } from './dist/index.js';

async function test() {
    console.log('Testing readap-js high-level API...');

    try {
        // Initialize the client (initialization is now automatic)
        const client = new DAPClient('https://example.com/data');

        console.log('✓ Client created successfully');

        // Test URL generation (these now handle initialization automatically)
        console.log('DAS URL:', await client.getDasUrl());
        console.log('DDS URL:', await client.getDdsUrl());
        console.log('DODS URL:', await client.getDodsUrl());

        // Test with simple constraints first
        console.log('Testing simple variable selection...');
        const simpleUrl = await client.getDodsUrl(['mean_wave_dir']);
        console.log('Simple variable URL:', simpleUrl);
        
        // Test with constraints
        console.log('Testing with constraints...');
        const constrainedUrl = await client.getDodsUrl(['mean_wave_dir'], { 
            mean_wave_dir: [{ start: 0, end: 5, stride: 2 }] 
        });
        console.log('Constrained URL:', constrainedUrl);

        // Test parsing
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

        const dasContent = `Attributes {
    time {
        String long_name "Epoch Time";
        String units "seconds since 1970-01-01 00:00:00 UTC";
    }
    mean_wave_dir {
        String long_name "Mean Wave Direction";
        String units "degrees_true";
        Int32 _FillValue 999;
    }
}`;

        console.log('\n=== Testing DDS Parsing ===');
        const dataset = await client.parseDds(ddsContent);
        console.log('Dataset:', dataset.name);
        console.log('Variables:', dataset.variables);

        console.log('\n=== Testing DAS Parsing ===');
        const attributes = await client.parseDas(dasContent);
        console.log('Attributes:', Object.keys(attributes));

        console.log('\n✓ All readap-js tests passed!');

    } catch (error) {
        console.error('✗ Error:', error.message);
        console.error('Full error:', error);
    }
}

test();