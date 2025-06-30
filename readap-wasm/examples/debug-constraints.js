#!/usr/bin/env bun
// Debug script to test constraint building
import init, { OpenDAPDataset, ConstraintBuilder } from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function debugConstraints() {
    console.log('🔍 Debugging constraint building...\n');
    
    await init();
    
    try {
        // Load dataset
        console.log('Loading dataset...');
        const dataset = await OpenDAPDataset.fromURL(BASE_URL);
        console.log('✅ Dataset loaded\n');
        
        // Test constraint builder directly
        console.log('Test 1: Direct constraint builder');
        const builder = new ConstraintBuilder();
        builder.isel({
            time: { type: "single", value: 0 },
            latitude: { type: "range", start: 10, end: 20 }
        });
        const constraint1 = builder.build();
        console.log(`   Constraint: "${constraint1}"`);
        
        // Test dataset selection constraint building
        console.log('\nTest 2: Dataset selection constraint');
        const selection = dataset.isel({
            time: { type: "single", value: 0 },
            latitude: { type: "range", start: 10, end: 20 }
        });
        // Try to access the constraint from the selection
        console.log('   Selection created');
        
        // Test different constraint patterns
        console.log('\nTest 3: Various constraint patterns');
        
        // Single indices
        const b1 = new ConstraintBuilder();
        b1.isel({ time: { type: "single", value: 0 } });
        console.log(`   Single index: "${b1.build()}"`);
        
        // Multiple singles
        const b2 = new ConstraintBuilder();
        b2.isel({
            time: { type: "single", value: 0 },
            step: { type: "single", value: 0 }
        });
        console.log(`   Multiple singles: "${b2.build()}"`);
        
        // Range
        const b3 = new ConstraintBuilder();
        b3.isel({
            latitude: { type: "range", start: 10, end: 20 }
        });
        console.log(`   Range: "${b3.build()}"`);
        
        // Full constraint for t2m
        const b4 = new ConstraintBuilder();
        b4.isel({
            longitude: { type: "single", value: 0 },
            latitude: { type: "range", start: 10, end: 20 },
            time: { type: "single", value: 0 },
            step: { type: "single", value: 0 }
        });
        console.log(`   Full t2m constraint: "${b4.build()}"`);
        
        // Test variable info to understand dimensions
        console.log('\nTest 4: Variable dimensions');
        const t2mInfo = JSON.parse(dataset.getVariableInfo('t2m'));
        console.log('   t2m dimensions:', t2mInfo.dimensions);
        
    } catch (error) {
        console.error('❌ Error:', error);
    }
}

debugConstraints();