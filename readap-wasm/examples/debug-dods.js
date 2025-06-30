#!/usr/bin/env bun
// Debug script to test DODS data fetching
import init, { OpenDAPDataset } from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function debugDODS() {
    console.log('🔍 Debugging DODS data fetching...\n');
    
    await init();
    
    try {
        // Load dataset
        console.log('Loading dataset...');
        const dataset = await OpenDAPDataset.fromURL(BASE_URL);
        console.log('✅ Dataset loaded\n');
        
        // Test simple variable fetch without constraints
        console.log('Test 1: Fetch single value (no constraints)');
        try {
            const selection = dataset.isel({
                time: { type: "single", value: 0 },
                latitude: { type: "single", value: 0 },
                longitude: { type: "single", value: 0 },
                step: { type: "single", value: 0 }
            });
            const data = await dataset.getVariable('t2m', selection);
            console.log('✅ Single value fetch succeeded');
            console.log(`   Data length: ${data.length}`);
            console.log(`   Data type: ${data.data.constructor.name}`);
            console.log(`   First value: ${data.data[0]}`);
        } catch (e) {
            console.log('❌ Single value fetch failed:', e.message);
        }
        
        // Test small range
        console.log('\nTest 2: Fetch small range');
        try {
            const selection = dataset.isel({
                time: { type: "single", value: 0 },
                latitude: { type: "range", start: 0, end: 2 },
                longitude: { type: "range", start: 0, end: 2 },
                step: { type: "single", value: 0 }
            });
            const data = await dataset.getVariable('t2m', selection);
            console.log('✅ Small range fetch succeeded');
            console.log(`   Data length: ${data.length}`);
        } catch (e) {
            console.log('❌ Small range fetch failed:', e.message);
        }
        
        // Test the exact constraint from the example
        console.log('\nTest 3: Fetch with example constraint (time 0, lat 10-20)');
        try {
            const selection = dataset.isel({
                time: { type: "single", value: 0 },
                latitude: { type: "range", start: 10, end: 20 }
            });
            const data = await dataset.getVariable('t2m', selection);
            console.log('✅ Example constraint fetch succeeded');
            console.log(`   Data length: ${data.length}`);
        } catch (e) {
            console.log('❌ Example constraint fetch failed:', e.message);
            console.log('   This is the failing case from bun-example.js');
        }
        
        // Test manual URL construction
        console.log('\nTest 4: Manual DODS URL test');
        const manualUrl = `${BASE_URL}.dods?t2m[0][0:2][0:2][0]`;
        console.log(`   URL: ${manualUrl}`);
        console.log('   Fetching with manual constraint...');
        
        try {
            const response = await fetch(manualUrl);
            console.log(`   Response status: ${response.status}`);
            console.log(`   Response type: ${response.headers.get('content-type')}`);
            const buffer = await response.arrayBuffer();
            console.log(`   Response size: ${buffer.byteLength} bytes`);
        } catch (e) {
            console.log('   Fetch failed:', e.message);
        }
        
    } catch (error) {
        console.error('❌ Error:', error);
    }
}

debugDODS();