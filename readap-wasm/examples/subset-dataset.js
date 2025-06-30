#!/usr/bin/env node

/**
 * subset-dataset.js - Geographic subsetting example using xarray-style coordinate selection
 * 
 * This script demonstrates the new XArray-style API that:
 * 1. Downloads coordinates upfront during dataset initialization
 * 2. Provides coordinate-based selection similar to xarray.Dataset.sel()
 * 3. Handles geographic bounding box selection automatically
 * 4. Shows temperature data extraction for the northeast US
 */

import init, { XArrayDataset } from '@mattnucc/readap';

const URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

// Northeast US bounding box (approximate)
const NORTHEAST_BBOX = {
    lon_min: -80.0,    // Western boundary (Ohio/Pennsylvania border) 
    lon_max: -66.0,    // Eastern boundary (Atlantic coast)
    lat_min: 38.0,     // Southern boundary (Virginia/North Carolina)
    lat_max: 47.0      // Northern boundary (Maine/Vermont border)
};

async function main() {
    console.log('=== XArray-Style OpenDAP Geographic Subsetting ===');
    console.log('Dataset:', URL);
    console.log('Target variable: t2m (2-meter temperature)');
    console.log('Geographic region: Northeast US');
    console.log(`Bounding box: ${NORTHEAST_BBOX.lat_min}°N-${NORTHEAST_BBOX.lat_max}°N, ${NORTHEAST_BBOX.lon_min}°W-${NORTHEAST_BBOX.lon_max}°W`);
    console.log('');

    try {
        // Initialize WASM
        console.log('1. Initializing WASM...');
        await init();
        console.log('✓ WASM initialized');
        console.log('');

        // Load dataset with automatic coordinate downloading
        console.log('2. Loading dataset and coordinates (xarray-style)...');
        const ds = await XArrayDataset.fromURL(URL);
        console.log('✓ Dataset loaded with coordinate indexing');
        console.log('');

        // Show available variables
        console.log('3. Available variables:');
        const varNames = ds.getVariableNames();
        console.log('  Variables:', Array.from(varNames));
        console.log('');

        // Show coordinate information
        console.log('4. Coordinate information:');
        const coordinates = JSON.parse(ds.getCoordinates());
        for (const [name, info] of Object.entries(coordinates)) {
            console.log(`  ${name}: ${info.size} points (${info.values[0]} to ${info.values[info.values.length-1]})`);
        }
        console.log('');

        // Geographic subsetting using coordinate values (xarray-style)
        console.log('5. Geographic subsetting with coordinate selection...');
        
        // Single point selection (closest to NYC)
        console.log('  Example 1: Single point near NYC (40.7°N, 74.0°W)');
        const nycData = await ds.sel('t2m', {
            latitude: 40.7,
            longitude: 360 - 74.0,  // Convert to 0-360 longitude system
            time: 0,      // First time step (will find nearest index)
            step: 0       // First forecast step
        });
        
        const nycTemp = Array.from(nycData.data)[0];
        console.log(`    Temperature: ${nycTemp.toFixed(2)}K (${(nycTemp - 273.15).toFixed(1)}°C, ${((nycTemp - 273.15) * 9/5 + 32).toFixed(1)}°F)`);
        console.log('');

        // Bounding box selection
        console.log('  Example 2: Northeast US bounding box');
        const bboxData = await ds.sel('t2m', {
            latitude: { min: NORTHEAST_BBOX.lat_min, max: NORTHEAST_BBOX.lat_max },
            longitude: { 
                min: 360 + NORTHEAST_BBOX.lon_min,  // Convert to 0-360 system
                max: 360 + NORTHEAST_BBOX.lon_max 
            },
            time: 0,      // First time step
            step: 0       // First forecast step
        });
        
        const bboxTemps = Array.from(bboxData.data);
        const avgTemp = bboxTemps.reduce((sum, val) => sum + val, 0) / bboxTemps.length;
        const minTemp = Math.min(...bboxTemps);
        const maxTemp = Math.max(...bboxTemps);
        
        console.log(`    Data points: ${bboxTemps.length}`);
        console.log(`    Temperature range: ${minTemp.toFixed(2)}K to ${maxTemp.toFixed(2)}K`);
        console.log(`    Temperature range (°C): ${(minTemp - 273.15).toFixed(1)}°C to ${(maxTemp - 273.15).toFixed(1)}°C`);
        console.log(`    Average temperature: ${avgTemp.toFixed(2)}K (${(avgTemp - 273.15).toFixed(1)}°C)`);
        console.log('');

        // Time series selection (single location, multiple times)
        console.log('  Example 3: Time series at Boston (42.3°N, 71.1°W) - first 5 time steps');
        const bostonTimeSeries = await ds.sel('t2m', {
            latitude: 42.3,
            longitude: 360 - 71.1,
            time: { min: 0, max: 4 },  // First 5 time steps
            step: 0
        });
        
        const timeSeriesTemps = Array.from(bostonTimeSeries.data);
        console.log('    Time series temperatures (°C):');
        timeSeriesTemps.forEach((temp, i) => {
            console.log(`      Time ${i}: ${(temp - 273.15).toFixed(1)}°C`);
        });
        console.log('');

        // Index-based selection (advanced users)
        console.log('6. Index-based selection (isel - for advanced users)...');
        const indexData = await ds.isel('t2m', {
            longitude: { min: 100, max: 200 },  // Direct index range
            latitude: { min: 200, max: 300 },
            time: 0,
            step: 0
        });
        
        const indexTemps = Array.from(indexData.data);
        console.log(`  Selected ${indexTemps.length} points using direct indices`);
        console.log(`  Temperature range: ${(Math.min(...indexTemps) - 273.15).toFixed(1)}°C to ${(Math.max(...indexTemps) - 273.15).toFixed(1)}°C`);
        console.log('');

        console.log('✅ XArray-style geographic subsetting completed!');
        console.log('');
        console.log('🎯 Key features demonstrated:');
        console.log('   • Automatic coordinate downloading and indexing');
        console.log('   • xarray-style .sel() method for coordinate-based selection');
        console.log('   • Geographic bounding box selection using lat/lon values');
        console.log('   • Single point selection with nearest-neighbor matching');
        console.log('   • Time series extraction at specific locations');
        console.log('   • Advanced index-based selection with .isel()');
        console.log('   • No manual coordinate-to-index mapping required!');

    } catch (error) {
        console.error('❌ Error:', error.message);
        console.error('Stack:', error.stack);
        process.exit(1);
    }
}

main().catch(err => {
    console.error('Unhandled error:', err);
    process.exit(1);
});