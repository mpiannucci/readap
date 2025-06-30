#!/usr/bin/env bun
// Test the new universal fetch abstraction across different runtimes

import init, { 
    UniversalFetch, 
    SimpleConstraintBuilder, 
    OpenDAPUrlBuilder,
    OpenDAPDataset,
    universalFetchText,
    universalFetchBinary
} from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function testUniversalFetch() {
    console.log('🌐 Testing Universal Fetch Abstraction');
    console.log('=' .repeat(50));
    
    await init();
    
    let passedTests = 0;
    let totalTests = 0;
    
    function test(description, testFn) {
        totalTests++;
        return testFn().then(result => {
            if (result) {
                console.log(`✅ ${description}`);
                passedTests++;
            } else {
                console.log(`❌ ${description} - Test returned false`);
            }
        }).catch(error => {
            console.log(`❌ ${description} - Error: ${error.message}`);
        });
    }
    
    // Test 1: Basic fetch client creation
    await test('Create UniversalFetch client', async () => {
        const fetch_client = new UniversalFetch();
        const runtime_info = fetch_client.getRuntimeInfo();
        console.log(`    Runtime Info: ${runtime_info}`);
        return runtime_info.length > 0;
    });
    
    // Test 2: DAS metadata fetch
    await test('Fetch DAS metadata with UniversalFetch', async () => {
        const fetch_client = new UniversalFetch();
        const das_url = `${BASE_URL}.das`;
        const das_text = await fetch_client.fetchText(das_url);
        
        console.log(`    DAS size: ${das_text.length} characters`);
        console.log(`    Contains: ${das_text.includes('Attributes') ? 'Attributes' : 'No Attributes'}`);
        
        return das_text.length > 0 && das_text.includes('Attributes');
    });
    
    // Test 3: DDS metadata fetch
    await test('Fetch DDS metadata with UniversalFetch', async () => {
        const fetch_client = new UniversalFetch();
        const dds_url = `${BASE_URL}.dds`;
        const dds_text = await fetch_client.fetchText(dds_url);
        
        console.log(`    DDS size: ${dds_text.length} characters`);
        console.log(`    Contains: ${dds_text.includes('Dataset') ? 'Dataset' : 'No Dataset'}`);
        
        return dds_text.length > 0 && dds_text.includes('Dataset');
    });
    
    // Test 4: DODS binary data fetch
    await test('Fetch DODS binary data with UniversalFetch', async () => {
        const fetch_client = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        
        const constraint = new SimpleConstraintBuilder()
            .addSingle('time', 0)
            .addSingle('latitude', 0)
            .addSingle('longitude', 0)
            .addSingle('step', 0)
            .build();
        
        const dods_url = url_builder.dodsUrl(`t2m[${constraint}]`);
        const binary_data = await fetch_client.fetchBinary(dods_url);
        
        console.log(`    DODS size: ${binary_data.length} bytes`);
        console.log(`    First 8 bytes: ${Array.from(binary_data.slice(0, 8)).map(b => b.toString(16).padStart(2, '0')).join(' ')}`);
        
        return binary_data.length > 0;
    });
    
    // Test 5: DODS as Uint8Array
    await test('Fetch DODS as Uint8Array', async () => {
        const fetch_client = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        
        const dods_url = url_builder.dodsUrl('t2m[0][0][0][0]');
        const uint8_array = await fetch_client.fetchBinaryAsArray(dods_url);
        
        console.log(`    Uint8Array length: ${uint8_array.length}`);
        
        return uint8_array.length > 0 && uint8_array instanceof Uint8Array;
    });
    
    // Test 6: Standalone convenience functions
    await test('Test standalone fetch functions', async () => {
        const das_url = `${BASE_URL}.das`;
        const das_text = await universalFetchText(das_url);
        
        const dods_url = `${BASE_URL}.dods?t2m[0][0][0][0]`;
        const binary_data = await universalFetchBinary(dods_url);
        
        console.log(`    Standalone DAS: ${das_text.length} chars`);
        console.log(`    Standalone DODS: ${binary_data.length} bytes`);
        
        return das_text.length > 0 && binary_data.length > 0;
    });
    
    // Test 7: Dataset with new fetch abstraction
    await test('OpenDAPDataset with UniversalFetch', async () => {
        const dataset = await OpenDAPDataset.fromURL(BASE_URL);
        const variables = dataset.getVariableNames();
        
        console.log(`    Variables loaded: ${variables.length}`);
        console.log(`    Sample variables: ${variables.slice(0, 3).join(', ')}`);
        
        return variables.length > 0 && variables.includes('t2m');
    });
    
    // Test 8: Custom headers test
    await test('Custom headers functionality', async () => {
        const fetch_client = new UniversalFetch();
        
        // Create custom headers object
        const headers = {};
        headers['X-Custom-Header'] = 'readap-test';
        headers['Accept'] = 'application/octet-stream';
        
        fetch_client.setDefaultHeaders(headers);
        
        // Test with a simple request
        const das_url = `${BASE_URL}.das`;
        const das_text = await fetch_client.fetchText(das_url);
        
        console.log(`    Custom headers set, DAS fetch: ${das_text.length > 0 ? 'success' : 'failed'}`);
        
        return das_text.length > 0;
    });
    
    // Test 9: Timeout functionality test
    await test('Timeout configuration', async () => {
        const fetch_client = new UniversalFetch();
        
        // Set a reasonable timeout (10 seconds)
        fetch_client.setTimeout(10000);
        
        // Test with a normal request
        const das_url = `${BASE_URL}.das`;
        const das_text = await fetch_client.fetchText(das_url);
        
        console.log(`    Timeout set, DAS fetch: ${das_text.length > 0 ? 'success' : 'failed'}`);
        
        return das_text.length > 0;
    });
    
    // Test 10: Performance comparison
    await test('Performance comparison test', async () => {
        const fetch_client = new UniversalFetch();
        const das_url = `${BASE_URL}.das`;
        
        // Time 3 sequential fetches
        const start_time = performance.now();
        
        for (let i = 0; i < 3; i++) {
            await fetch_client.fetchText(das_url);
        }
        
        const end_time = performance.now();
        const total_time = end_time - start_time;
        const avg_time = total_time / 3;
        
        console.log(`    3 sequential fetches: ${total_time.toFixed(2)}ms total`);
        console.log(`    Average per fetch: ${avg_time.toFixed(2)}ms`);
        
        return avg_time < 5000; // Should be reasonable (under 5 seconds per fetch)
    });
    
    // Summary
    console.log('\\n📊 Universal Fetch Test Summary');
    console.log('=' .repeat(50));
    console.log(`Passed: ${passedTests}/${totalTests} tests`);
    console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
    
    if (passedTests === totalTests) {
        console.log('🎉 All universal fetch tests passed!');
        console.log('✅ Runtime-agnostic fetch abstraction working correctly');
        console.log('✅ Compatible with current JavaScript runtime');
        console.log('✅ DAS, DDS, and DODS data fetching all functional');
        console.log('✅ Dataset integration working with new fetch system');
        return true;
    } else {
        console.log('⚠️  Some universal fetch tests failed. Issues identified:');
        if (passedTests < totalTests) {
            console.log('   - Check network connectivity');
            console.log('   - Verify runtime compatibility');
            console.log('   - Check fetch implementation for this environment');
        }
        return false;
    }
}

// Run tests
testUniversalFetch()
    .then(success => {
        console.log(`\\n${success ? '🎉 Universal fetch tests completed successfully!' : '❌ Universal fetch tests failed'}`);
        process.exit(success ? 0 : 1);
    })
    .catch(error => {
        console.error('❌ Universal fetch test runner failed:', error);
        process.exit(1);
    });