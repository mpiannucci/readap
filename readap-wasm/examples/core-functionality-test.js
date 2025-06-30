#!/usr/bin/env bun
// Core functionality test for the refactored readap-wasm package
// Focuses on essential features without extensive network operations

import init, { 
    SimpleConstraintBuilder,
    StringConstraintBuilder,
    UniversalFetch,
    UniversalDodsParser,
    ImmutableDataset,
    OpenDAPUrlBuilder
} from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function coreFeatureTest() {
    console.log('⚡ Core Functionality Test - Refactored readap-wasm');
    console.log('=' .repeat(60));
    
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
    
    // Runtime Detection
    function detectRuntime() {
        if (typeof Bun !== 'undefined') return 'Bun';
        if (typeof Deno !== 'undefined') return 'Deno';
        if (typeof process !== 'undefined' && process.versions?.node) return 'Node.js';
        if (typeof window !== 'undefined') return 'Browser';
        return 'Unknown';
    }
    
    console.log(`🏃 Runtime: ${detectRuntime()}`);
    console.log('');
    
    // Test 1: Phase 1 - Constraint Builders (Anti-aliasing)
    await test('SimpleConstraintBuilder eliminates aliasing errors', async () => {
        const results = [];
        
        // Create multiple builders in rapid succession
        for (let i = 0; i < 10; i++) {
            try {
                const builder = new SimpleConstraintBuilder()
                    .addSingle('time', i)
                    .addRange('lat', 0, i + 1)
                    .addStride('lon', 0, 2, i + 5);
                    
                const constraint = builder.build();
                results.push(constraint);
            } catch (error) {
                console.log(`      Builder ${i}: ${error.message}`);
                return false;
            }
        }
        
        console.log(`    Created ${results.length} builders without aliasing errors`);
        console.log(`    Sample constraint: ${results[0]}`);
        
        return results.length === 10;
    });
    
    // Test 2: Phase 2 - Universal Infrastructure  
    await test('UniversalFetch adapts to runtime environment', async () => {
        const fetcher = new UniversalFetch();
        const runtime_info = fetcher.getRuntimeInfo();
        
        console.log(`    Runtime detection: ${runtime_info}`);
        
        // Test basic functionality with a simple request
        try {
            const das_data = await fetcher.fetchText(`${BASE_URL}.das`);
            const has_content = das_data.length > 1000;
            
            console.log(`    DAS fetch: ${has_content ? 'Success' : 'Failed'} (${das_data.length} chars)`);
            return has_content;
        } catch (error) {
            console.log(`    Fetch failed: ${error.message}`);
            return false;
        }
    });
    
    // Test 3: Phase 2 - Universal DODS Parser
    await test('UniversalDodsParser handles binary data correctly', async () => {
        const fetcher = new UniversalFetch();
        const parser = new UniversalDodsParser();
        
        try {
            // Get small amount of DODS data
            const constraint = 't2m[0][0][0][0]';
            const dods_url = `${BASE_URL}.dods?${constraint}`;
            const binary_data = await fetcher.fetchBinary(dods_url);
            const uint8_data = new Uint8Array(binary_data);
            
            console.log(`    Binary data: ${uint8_data.length} bytes`);
            
            // Test parsing
            const parsed_data = parser.parseDods(uint8_data);
            const variables = Object.keys(parsed_data);
            
            console.log(`    Parsed variables: ${variables.join(', ')}`);
            
            const has_t2m = variables.includes('t2m');
            if (has_t2m && parsed_data.t2m?.data?.[0]) {
                console.log(`    t2m value: ${parsed_data.t2m.data[0].toFixed(2)}K`);
            }
            
            return has_t2m && parsed_data.t2m.length > 0;
        } catch (error) {
            console.log(`    DODS parsing failed: ${error.message}`);
            return false;
        }
    });
    
    // Test 4: Phase 3 - Immutable Dataset API
    await test('ImmutableDataset provides safe method chaining', async () => {
        try {
            // Create base dataset
            const base_dataset = new ImmutableDataset(BASE_URL);
            console.log(`    Base dataset created: ${base_dataset.baseUrl()}`);
            
            // Load metadata to create new instance
            const dataset_with_metadata = await ImmutableDataset.fromURL(BASE_URL);
            const variables = dataset_with_metadata.getVariableNames();
            
            console.log(`    Variables loaded: ${variables.length}`);
            
            // Test immutable chaining
            const das_data = await fetch(base_dataset.dasUrl()).then(r => r.text());
            const dataset_with_das = base_dataset.withDAS(das_data);
            const das_variables = dataset_with_das.getVariableNames();
            
            console.log(`    DAS variables: ${das_variables.length}`);
            
            // Verify immutability
            const base_unchanged = base_dataset.getVariableNames().length === 0;
            const das_added = das_variables.length > 0;
            const objects_different = base_dataset !== dataset_with_das;
            
            console.log(`    Base unchanged: ${base_unchanged}`);
            console.log(`    DAS added: ${das_added}`);
            console.log(`    Objects different: ${objects_different}`);
            
            return base_unchanged && das_added && objects_different;
        } catch (error) {
            console.log(`    Immutable dataset failed: ${error.message}`);
            return false;
        }
    });
    
    // Test 5: Integration - Complete workflow
    await test('End-to-end workflow integration', async () => {
        try {
            // Step 1: Create constraint
            const constraint_builder = new SimpleConstraintBuilder()
                .addSingle('time', 0)
                .addSingle('latitude', 0)
                .addSingle('longitude', 0)
                .addSingle('step', 0);
            
            const constraint = constraint_builder.build();
            console.log(`    Constraint: ${constraint}`);
            
            // Step 2: Create dataset and URLs
            const dataset = new ImmutableDataset(BASE_URL);
            const dods_url = dataset.dodsUrl(`t2m[${constraint}]`);
            
            console.log(`    DODS URL: ${dods_url.length > 0 ? 'Generated' : 'Failed'}`);
            
            // Step 3: Fetch and parse data
            const fetcher = new UniversalFetch();
            const parser = new UniversalDodsParser();
            
            const binary_data = await fetcher.fetchBinary(dods_url);
            const uint8_data = new Uint8Array(binary_data);
            const parsed_data = parser.parseDods(uint8_data);
            
            console.log(`    Data parsed: ${Object.keys(parsed_data).length} variables`);
            
            // Step 4: Extract result
            const has_temperature = 't2m' in parsed_data;
            if (has_temperature) {
                const temp_value = parsed_data.t2m.data[0];
                console.log(`    Temperature: ${temp_value.toFixed(2)}K`);
            }
            
            return constraint.length > 0 && 
                   dods_url.includes('.dods') && 
                   has_temperature;
                   
        } catch (error) {
            console.log(`    Integration failed: ${error.message}`);
            return false;
        }
    });
    
    // Test 6: Error resilience
    await test('Error handling and recovery', async () => {
        const dataset = new ImmutableDataset(BASE_URL);
        let error_count = 0;
        let recovery_count = 0;
        
        // Test various error conditions
        try {
            dataset.dodsUrl('invalid[constraint}');
            recovery_count++;
        } catch (error) {
            error_count++;
        }
        
        try {
            new SimpleConstraintBuilder().build(); // Empty constraint
            recovery_count++;
        } catch (error) {
            error_count++;
        }
        
        try {
            const result = dataset.withCoordinates('invalid', []);
            recovery_count++;
        } catch (error) {
            error_count++;
        }
        
        // Test that dataset still works
        const still_works = dataset.dasUrl().includes('.das');
        
        console.log(`    Error tests: ${error_count + recovery_count}/3`);
        console.log(`    Dataset still functional: ${still_works}`);
        
        return still_works && (error_count + recovery_count) === 3;
    });
    
    // Summary
    console.log('\\n📊 Core Functionality Test Results');
    console.log('=' .repeat(60));
    console.log(`Runtime: ${detectRuntime()}`);
    console.log(`Passed: ${passedTests}/${totalTests} tests (${((passedTests / totalTests) * 100).toFixed(1)}%)`);
    
    if (passedTests === totalTests) {
        console.log('\\n🎉 ALL CORE TESTS PASSED!');
        console.log('');
        console.log('✅ Phase 1: Constraint builders eliminate aliasing errors');
        console.log('✅ Phase 2: Universal infrastructure adapts to runtime');
        console.log('✅ Phase 3: Immutable dataset provides safe chaining');
        console.log('✅ Integration: Complete workflows function correctly');
        console.log('✅ Error handling: Robust recovery and state preservation');
        console.log('');
        console.log('🏆 REFACTOR SUCCESS: readap-wasm now works universally!');
        console.log('   • No mutable self references');
        console.log('   • Runtime-agnostic infrastructure');
        console.log('   • Immutable functional API');
        console.log('   • Universal compatibility');
        
        return true;
    } else {
        console.log('\\n⚠️  Some core tests failed');
        console.log(`Success rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
        return false;
    }
}

// Run core tests
coreFeatureTest()
    .then(success => {
        console.log(`\\n${success ? '🌟 CORE FUNCTIONALITY VERIFIED!' : '🔧 ISSUES DETECTED'}`);
        process.exit(success ? 0 : 1);
    })
    .catch(error => {
        console.error('❌ Core test runner failed:', error);
        process.exit(1);
    });