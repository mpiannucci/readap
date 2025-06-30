#!/usr/bin/env bun
// Example Bun script demonstrating readap-wasm for OpenDAP data analysis
// Run with: bun run bun-example.js

import init, { OpenDAPDataset } from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function analyzeMeteorologyData() {
    console.log('🌊 ReadAP WASM + Bun Example - Meteorological Data Analysis');
    console.log('='.repeat(60));
    
    // Initialize the WASM module
    await init();
    
    try {
        // Load the dataset
        console.log('📡 Loading dataset metadata...');
        const dataset = await OpenDAPDataset.fromURL(BASE_URL);
        
        // Display available variables
        const variables = dataset.getVariableNames();
        console.log(`📊 Found ${variables.length} variables:`);
        variables.forEach((name, i) => {
            console.log(`  ${i + 1}. ${name}`);
        });
        
        // Analyze a specific variable
        console.log('\n🔍 Analyzing temperature data (t2m)...');
        const tempInfo = JSON.parse(dataset.getVariableInfo('t2m'));
        console.log(`   - Dimensions: ${tempInfo.dimensions?.join(', ') || 'N/A'}`);
        console.log(`   - Attributes: ${Object.keys(tempInfo.attributes || {}).length} found`);
        
        // Load coordinates for advanced selections
        console.log('\n📍 Loading coordinate data...');
        const coords = ['time', 'latitude', 'longitude'];
        await Promise.all(coords.map(coord => dataset.loadCoordinates(coord)));
        console.log('   ✅ Coordinates loaded successfully');
        
        // Perform different types of data selections
        console.log('\n🎯 Performing data selections...');
        
        // 1. Simple index-based selection
        console.log('   → Index-based selection (first time, lat 10-20)');
        const indexSelection = dataset.isel({
            time: { type: "single", value: 0 },
            latitude: { type: "range", start: 10, end: 20 }
        });
        const tempIndexData = await dataset.getVariable('t2m', indexSelection);
        console.log(`     Data shape: ${tempIndexData.length} elements`);
        console.log(`     Temperature range: ${Math.min(...tempIndexData.data)} to ${Math.max(...tempIndexData.data)} K`);
        
        // 2. Value-based selection with nearest neighbor
        console.log('   → Value-based selection (NYC area)');
        const valueSelection = dataset.sel({
            latitude: [40.0, 41.0],    // NYC latitude range
            longitude: [-75.0, -73.0]  // NYC longitude range
        });
        const tempValueData = await dataset.getVariable('t2m', valueSelection);
        console.log(`     Data shape: ${tempValueData.length} elements`);
        
        // 3. Multi-variable analysis
        console.log('   → Multi-variable analysis');
        const multiVars = ['t2m', 'tcc', 'gust'];
        const multiData = await dataset.getVariables(multiVars, indexSelection);
        
        console.log('     Variable statistics:');
        Object.entries(multiData).forEach(([varName, data]) => {
            const min = Math.min(...data.data);
            const max = Math.max(...data.data);
            const avg = data.data.reduce((a, b) => a + b, 0) / data.data.length;
            console.log(`       ${varName}: min=${min.toFixed(2)}, max=${max.toFixed(2)}, avg=${avg.toFixed(2)}`);
        });
        
        // 4. Chained selections for complex analysis
        console.log('   → Chained selection (surface data for specific region)');
        const chainedSelection = dataset
            .isel({ time: { type: "single", value: 0 } })
            .sel({ latitude: [35.0, 45.0], longitude: [-80.0, -70.0] });
        
        const surfaceTemp = await dataset.getVariable('t2m', chainedSelection);
        console.log(`     Surface temperature data: ${surfaceTemp.length} grid points`);
        
        // Performance timing
        console.log('\n⏱️  Performance test - rapid data access:');
        const startTime = performance.now();
        
        const rapidSelections = await Promise.all([
            dataset.getVariable('t2m', dataset.isel({ time: { type: "single", value: 0 } })),
            dataset.getVariable('tcc', dataset.isel({ time: { type: "single", value: 0 } })),
            dataset.getVariable('gust', dataset.isel({ time: { type: "single", value: 0 } }))
        ]);
        
        const endTime = performance.now();
        console.log(`   ✅ Fetched 3 variables in ${(endTime - startTime).toFixed(2)}ms`);
        
        // Summary
        console.log('\n📈 Analysis Summary:');
        console.log(`   • Dataset variables: ${variables.length}`);
        console.log(`   • Total data points analyzed: ${rapidSelections.reduce((sum, data) => sum + data.length, 0)}`);
        console.log(`   • Coordinate systems: ${coords.length} loaded`);
        console.log('   • Selection methods: index-based, value-based, chained');
        
    } catch (error) {
        console.error('❌ Error during analysis:', error);
        process.exit(1);
    }
}

// Run the analysis
console.log('🚀 Starting Bun + ReadAP WASM analysis...\n');
analyzeMeteorologyData()
    .then(() => {
        console.log('\n✅ Analysis completed successfully!');
        process.exit(0);
    })
    .catch((error) => {
        console.error('\n❌ Analysis failed:', error);
        process.exit(1);
    });
