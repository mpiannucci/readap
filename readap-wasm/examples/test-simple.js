#!/usr/bin/env bun
// Simple test to isolate the issue
import init, { OpenDAPDataset } from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function testSimple() {
    console.log('Testing simple data fetch...\n');
    
    await init();
    
    try {
        // Test 1: Direct fetch without using the high-level API
        console.log('Test 1: Manual DODS fetch');
        const url = `${BASE_URL}.dods?t2m[0][0][0][0]`;
        const response = await fetch(url);
        const buffer = await response.arrayBuffer();
        console.log(`✅ Manual fetch succeeded: ${buffer.byteLength} bytes`);
        
        // Test 2: Load dataset
        console.log('\nTest 2: Load dataset');
        const dataset = await OpenDAPDataset.fromURL(BASE_URL);
        console.log('✅ Dataset loaded');
        
        // Test 3: Get variable info
        console.log('\nTest 3: Variable info');
        const info = dataset.getVariableInfo('t2m');
        console.log('✅ Variable info:', info);
        
        // Test 4: Try to get data without selection (should fail)
        console.log('\nTest 4: Get data without selection');
        try {
            const data = await dataset.getVariable('t2m');
            console.log('✅ Got data:', data.length);
        } catch (e) {
            console.log('❌ Expected error:', e.message || e);
        }
        
        // Test 5: Build URL manually
        console.log('\nTest 5: URL building');
        console.log('DAS URL:', dataset.dasUrl());
        console.log('DDS URL:', dataset.ddsUrl());
        // Try to get DODS URL with simple constraint
        console.log('DODS URL:', dataset.dodsUrl('t2m[0][0][0][0]'));
        
    } catch (error) {
        console.error('❌ Error:', error);
        console.error('Stack:', error.stack);
    }
}

testSimple();