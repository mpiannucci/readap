#!/usr/bin/env bun
// Debug script to test DAS parsing with actual URL data
import init, { OpenDAPDataset } from '../pkg/readap_wasm.js';

async function debugDAS() {
    await init();
    
    const url = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';
    
    try {
        console.log('Testing DAS parsing...');
        
        // Fetch DAS data manually
        const dasUrl = `${url}.das`;
        console.log(`Fetching DAS from: ${dasUrl}`);
        const dasResponse = await fetch(dasUrl);
        const dasData = await dasResponse.text();
        
        console.log('DAS data length:', dasData.length);
        console.log('First 200 chars:', dasData.substring(0, 200));
        
        // Try to parse with the WASM library
        console.log('\nTrying to parse DAS with WASM...');
        const dataset = OpenDAPDataset.fromDAS(dasData);
        console.log('✅ DAS parsing successful!');
        console.log('Variables:', dataset.getVariableNames());
        
    } catch (error) {
        console.error('❌ DAS parsing failed:', error);
        console.error('Error details:', error.message);
    }
}

debugDAS();