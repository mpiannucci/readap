#!/usr/bin/env bun
// Working example for Bun that avoids the problematic APIs
import init, { OpenDAPDataset } from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function runExample() {
    console.log('🌊 ReadAP WASM + Bun - Working Example');
    console.log('='.repeat(50));
    
    await init();
    
    try {
        // Load dataset
        console.log('\n📡 Loading dataset metadata...');
        const dataset = await OpenDAPDataset.fromURL(BASE_URL);
        
        // Display available variables
        const variables = dataset.getVariableNames();
        console.log(`\n📊 Found ${variables.length} variables:`);
        variables.forEach((name, i) => {
            const info = JSON.parse(dataset.getVariableInfo(name));
            console.log(`  ${i + 1}. ${name} (${info.data_type}) - dims: ${info.dimensions.join(', ')}`);
        });
        
        // Analyze dataset structure
        console.log('\n📐 Dataset Structure:');
        const t2mInfo = JSON.parse(dataset.getVariableInfo('t2m'));
        console.log('  Temperature (t2m) dimensions:', t2mInfo.dimensions);
        console.log('  Data type:', t2mInfo.data_type);
        
        // Manual data fetching (bypassing the problematic APIs)
        console.log('\n📥 Fetching sample data points...');
        
        // Fetch a single point
        console.log('\n1. Single point:');
        const singlePointUrl = dataset.dodsUrl('t2m[0][0][0][0]');
        const response1 = await fetch(singlePointUrl);
        const buffer1 = await response1.arrayBuffer();
        console.log(`   URL: ${singlePointUrl}`);
        console.log(`   Response size: ${buffer1.byteLength} bytes`);
        
        // Fetch a small slice
        console.log('\n2. Small 2D slice:');
        const sliceUrl = dataset.dodsUrl('t2m[0:2][0:2][0][0]');
        const response2 = await fetch(sliceUrl);
        const buffer2 = await response2.arrayBuffer();
        console.log(`   URL: ${sliceUrl}`);
        console.log(`   Response size: ${buffer2.byteLength} bytes`);
        
        // Fetch multiple variables
        console.log('\n3. Multiple variables:');
        const multiUrl = dataset.dodsUrl('t2m[0][0][0][0],tcc[0][0][0][0],gust[0][0][0][0]');
        const response3 = await fetch(multiUrl);
        const buffer3 = await response3.arrayBuffer();
        console.log(`   URL: ${multiUrl}`);
        console.log(`   Response size: ${buffer3.byteLength} bytes`);
        
        // Parse DODS binary data manually
        console.log('\n🔬 Parsing DODS data...');
        console.log('   Note: Full DODS parsing requires fixing the recursive object issue');
        console.log('   For now, use manual fetch and binary parsing as shown above');
        
        // Summary
        console.log('\n📈 Summary:');
        console.log('   ✅ Dataset metadata loading works');
        console.log('   ✅ URL building works');
        console.log('   ✅ Manual DODS fetching works');
        console.log('   ⚠️  Constraint builder has issues with Bun');
        console.log('   ⚠️  DODS parsing has issues with Bun');
        console.log('\n💡 Workaround: Use manual URL building and fetch for now');
        
    } catch (error) {
        console.error('\n❌ Error:', error);
    }
}

// Run the example
runExample()
    .then(() => console.log('\n✅ Example completed!'))
    .catch(error => console.error('\n❌ Failed:', error));