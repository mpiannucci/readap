#!/usr/bin/env bun
// Debug script to isolate the coordinate loading issue
import init, { OpenDAPDataset } from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function debugCoordinates() {
    console.log('🔍 Debugging coordinate loading issue...\n');
    
    await init();
    
    try {
        // Load dataset
        console.log('Loading dataset...');
        const dataset = await OpenDAPDataset.fromURL(BASE_URL);
        console.log('✅ Dataset loaded');
        
        // Try loading coordinates one by one
        console.log('\nLoading coordinates individually:');
        
        // Test 1: Load time coordinate
        console.log('1. Loading time coordinate...');
        try {
            await dataset.loadCoordinates('time');
            console.log('   ✅ time loaded successfully');
        } catch (e) {
            console.log('   ❌ time failed:', e.message);
        }
        
        // Test 2: Load latitude coordinate
        console.log('2. Loading latitude coordinate...');
        try {
            await dataset.loadCoordinates('latitude');
            console.log('   ✅ latitude loaded successfully');
        } catch (e) {
            console.log('   ❌ latitude failed:', e.message);
        }
        
        // Test 3: Load longitude coordinate
        console.log('3. Loading longitude coordinate...');
        try {
            await dataset.loadCoordinates('longitude');
            console.log('   ✅ longitude loaded successfully');
        } catch (e) {
            console.log('   ❌ longitude failed:', e.message);
        }
        
        // Test 4: Try Promise.all (this is where it might fail)
        console.log('\n4. Loading all coordinates with Promise.all...');
        try {
            await Promise.all([
                dataset.loadCoordinates('time'),
                dataset.loadCoordinates('latitude'),
                dataset.loadCoordinates('longitude')
            ]);
            console.log('   ✅ Promise.all succeeded');
        } catch (e) {
            console.log('   ❌ Promise.all failed:', e.message);
            console.log('   This is likely the recursive object issue');
        }
        
        // Test 5: Try sequential loading
        console.log('\n5. Loading coordinates sequentially...');
        try {
            for (const coord of ['time', 'latitude', 'longitude']) {
                console.log(`   Loading ${coord}...`);
                await dataset.loadCoordinates(coord);
            }
            console.log('   ✅ Sequential loading succeeded');
        } catch (e) {
            console.log('   ❌ Sequential loading failed:', e.message);
        }
        
    } catch (error) {
        console.error('❌ Error:', error);
    }
}

debugCoordinates();