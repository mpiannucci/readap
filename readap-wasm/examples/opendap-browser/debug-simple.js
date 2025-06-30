#!/usr/bin/env node

/**
 * debug-simple.js - Simple debug script to test basic OpenDAP operations
 * 
 * This script tests the basic functionality of @mattnucc/readap by:
 * 1. Loading a dataset
 * 2. Fetching latitude coordinate data
 * 3. Printing the results
 */

import init, { ImmutableDataset, SimpleConstraintBuilder } from '@mattnucc/readap';

const URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function main() {
    console.log('=== Simple OpenDAP Debug Script ===');
    console.log('URL:', URL);
    console.log('');

    try {
        // Initialize WASM
        console.log('1. Initializing WASM...');
        await init();
        console.log('✓ WASM initialized');
        console.log('');

        // Load dataset
        console.log('2. Loading dataset...');
        const dataset = await ImmutableDataset.fromURL(URL);
        console.log('✓ Dataset loaded');
        console.log('');

        // Get variable names
        console.log('3. Getting variable names...');
        const varNames = dataset.getVariableNames();
        console.log('✓ Variable names:', varNames);
        console.log('');

        // Get variables info
        console.log('4. Getting variables info...');
        const variablesInfoJson = dataset.getVariablesInfo();
        const variablesInfo = JSON.parse(variablesInfoJson);
        console.log('✓ Variables info loaded');
        
        // Print all variables info to see the structure
        console.log('All variables:');
        for (const [name, info] of Object.entries(variablesInfo)) {
            console.log(`  ${name}:`);
            console.log(`    Type: ${info.data_type}`);
            console.log(`    Dimensions: ${JSON.stringify(info.dimensions)}`);
            console.log(`    Attributes: ${Object.keys(info.attributes || {}).length} attributes`);
        }
        console.log('');

        // Check DDS directly
        console.log('DDS URL:', dataset.ddsUrl());
        console.log('DAS URL:', dataset.dasUrl());
        console.log('');

        // Test data fetching with timeout
        console.log('5. Testing data fetching...');
        
        try {
            // Approach 1: Fetch with timeout
            console.log('Approach 1: Fetching latitude with 10s timeout...');
            const fetchPromise = dataset.getVariable('latitude');
            const timeoutPromise = new Promise((_, reject) => 
                setTimeout(() => reject(new Error('Timeout after 10 seconds')), 10000)
            );
            
            const latData1 = await Promise.race([fetchPromise, timeoutPromise]);
            console.log('✓ Fetched latitude data (all):');
            console.log('  Data type:', typeof latData1.data);
            console.log('  Data length:', latData1.data.length);
            console.log('  First 5 values:', Array.from(latData1.data.slice(0, 5)));
            console.log('  Last 5 values:', Array.from(latData1.data.slice(-5)));
            console.log('');
        } catch (err) {
            console.error('✗ Approach 1 failed:', err.message);
        }

        try {
            // Approach 2: Fetch with simple constraint
            console.log('Approach 2: Fetching latitude with constraint latitude[0:4]...');
            const latData2 = await dataset.getVariable('latitude', 'latitude[0:4]');
            console.log('✓ Fetched latitude data (constrained):');
            console.log('  Data type:', typeof latData2.data);
            console.log('  Data length:', latData2.data.length);
            console.log('  Values:', Array.from(latData2.data));
            console.log('');
        } catch (err) {
            console.error('✗ Approach 2 failed:', err.message);
        }

        try {
            // Approach 3: Fetch with SimpleConstraintBuilder
            console.log('Approach 3: Using SimpleConstraintBuilder...');
            let builder = new SimpleConstraintBuilder();
            builder = builder.addRange('latitude', 0, 4);
            const constraint = builder.build();
            console.log('  Built constraint:', constraint);
            
            const latData3 = await dataset.getVariable('latitude', constraint);
            console.log('✓ Fetched latitude data (builder):');
            console.log('  Data type:', typeof latData3.data);
            console.log('  Data length:', latData3.data.length);
            console.log('  Values:', Array.from(latData3.data));
            console.log('');
        } catch (err) {
            console.error('✗ Approach 3 failed:', err.message);
        }

        try {
            // Approach 4: Single index
            console.log('Approach 4: Single index with SimpleConstraintBuilder...');
            let builder = new SimpleConstraintBuilder();
            builder = builder.addSingle('latitude', 0);
            const constraint = builder.build();
            console.log('  Built constraint:', constraint);
            
            const latData4 = await dataset.getVariable('latitude', constraint);
            console.log('✓ Fetched latitude data (single):');
            console.log('  Data type:', typeof latData4.data);
            console.log('  Data length:', latData4.data.length);
            console.log('  Values:', Array.from(latData4.data));
            console.log('');
        } catch (err) {
            console.error('✗ Approach 4 failed:', err.message);
        }

        try {
            // Approach 5: Multi-variable constraint (like what's failing in the browser)
            console.log('Approach 5: Multi-variable constraint...');
            let builder = new SimpleConstraintBuilder();
            builder = builder.addRange('longitude', 0, 10);
            builder = builder.addRange('latitude', 0, 10);
            builder = builder.addRange('time', 1, 2);
            builder = builder.addRange('step', 1, 2);
            const constraint = builder.build();
            console.log('  Built constraint:', constraint);
            console.log('  Expected: longitude[0:10],latitude[0:10],time[1:2],step[1:2]');
            
            // Enable debug mode for DODS parser
            console.log('  Enabling debug mode and testing...');
            const dodsUrl = dataset.dodsUrl(constraint);
            console.log('  DODS URL:', dodsUrl);
            
            // Try to fetch a gridded variable that uses these dimensions
            console.log('  Attempting to fetch gust with constraint...');
            const gridData = await dataset.getVariable('gust', constraint);
            console.log('  gridData result:', gridData);
            
            if (gridData && gridData.data) {
                console.log('✓ Fetched gridded data:');
                console.log('  Data type:', typeof gridData.data);
                console.log('  Data length:', gridData.data.length);
                console.log('  First 5 values:', Array.from(gridData.data.slice(0, 5)));
            } else {
                console.log('✗ Grid data is undefined or has no data property');
                console.log('  gridData:', gridData);
            }
            console.log('');
        } catch (err) {
            console.error('✗ Approach 5 failed:', err.message);
            console.error('  Full error:', err);
        }

        try {
            // Approach 6: Test direct DODS URL construction and parsing
            console.log('Approach 6: Direct DODS URL test and parsing...');
            
            // Construct URL manually like the working curl command
            const dodsUrl = `${URL}.dods?gust[0:1][0:1][0:1][0:1]`;
            console.log('  DODS URL:', dodsUrl);
            
            // Use the universal fetch to get the data directly
            const response = await fetch(dodsUrl);
            const arrayBuffer = await response.arrayBuffer();
            console.log('✓ Direct DODS fetch successful:');
            console.log('  Response status:', response.status);
            console.log('  Content-Type:', response.headers.get('content-type'));
            console.log('  Data length:', arrayBuffer.byteLength, 'bytes');
            
            // Try to parse this data with our parser
            const uint8Data = new Uint8Array(arrayBuffer);
            const parsedData = dataset.parseDODS(uint8Data);
            console.log('  Parsed data:', parsedData);
            
            // Check what variables we found
            if (parsedData) {
                const keys = Object.keys(parsedData);
                console.log('  Found variables:', keys);
                if (keys.includes('gust')) {
                    console.log('  ✓ gust variable found in parsed data!');
                    console.log('  gust data:', parsedData.gust);
                } else {
                    console.log('  ✗ gust variable NOT found in parsed data');
                }
            }
            console.log('');
        } catch (err) {
            console.error('✗ Approach 6 failed:', err.message);
        }

        console.log('=== Debug Complete ===');

    } catch (error) {
        console.error('Fatal error:', error.message);
        console.error('Stack:', error.stack);
        process.exit(1);
    }
}

main().catch(err => {
    console.error('Unhandled error:', err);
    process.exit(1);
});