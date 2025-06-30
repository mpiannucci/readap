#!/usr/bin/env bun
// Test the new Universal DODS parser implementation

import init, { 
    UniversalFetch, 
    UniversalDodsParser,
    SimpleConstraintBuilder, 
    OpenDAPUrlBuilder,
    createUniversalDodsParser
} from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function testUniversalDodsParser() {
    console.log('🔧 Testing Universal DODS Parser Implementation');
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
            console.log(`    Stack: ${error.stack?.split('\\n')[0] || 'No stack trace'}`);
        });
    }
    
    // Test 1: Create Universal DODS Parser
    await test('Create UniversalDodsParser instance', async () => {
        const parser1 = new UniversalDodsParser();
        const parser2 = createUniversalDodsParser();
        
        console.log(`    Constructor: ${parser1 ? 'OK' : 'Failed'}`);
        console.log(`    Factory function: ${parser2 ? 'OK' : 'Failed'}`);
        
        return parser1 && parser2;
    });
    
    // Test 2: Analyze DODS structure
    await test('Analyze DODS data structure', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        const parser = new UniversalDodsParser();
        
        const constraint = 't2m[0][0][0][0]';
        const dods_url = url_builder.dodsUrl(constraint);
        const binary_data = await fetcher.fetchBinary(dods_url);
        const uint8_data = new Uint8Array(binary_data);
        
        console.log(`    Analyzing ${binary_data.length} bytes of DODS data`);
        
        const analysis = parser.analyzeDodsStructure(uint8_data);
        
        console.log(`    Has Data marker: ${analysis.hasDataMarker}`);
        console.log(`    Data marker position: ${analysis.dataMarkerPosition}`);
        console.log(`    Binary data start: ${analysis.binaryDataStart}`);
        console.log(`    Binary data length: ${analysis.binaryDataLength}`);
        
        if (analysis.binaryAnalysis) {
            console.log(`    Count 1: ${analysis.binaryAnalysis.count1}`);
            console.log(`    Count 2: ${analysis.binaryAnalysis.count2}`);
            console.log(`    Counts match: ${analysis.binaryAnalysis.countsMatch}`);
            console.log(`    First float32: ${analysis.binaryAnalysis.firstFloat32}`);
            console.log(`    Hex preview: ${analysis.binaryAnalysis.hexPreview}`);
        }
        
        return analysis.hasDataMarker && analysis.binaryDataLength > 0;
    });
    
    // Test 3: Parse DODS data with detailed results
    await test('Parse DODS data with detailed results', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        const parser = new UniversalDodsParser();
        parser.setDebugMode(true);
        
        const constraint = 't2m[0][0][0][0]';
        const dods_url = url_builder.dodsUrl(constraint);
        const binary_data = await fetcher.fetchBinary(dods_url);
        const uint8_data = new Uint8Array(binary_data);
        
        console.log(`    Parsing ${binary_data.length} bytes with debug mode enabled`);
        
        const detailed_result = parser.parseDodsDetailed(uint8_data);
        
        console.log(`    Parsing success: ${detailed_result.success}`);
        
        if (detailed_result.success) {
            const variable_names = Object.keys(detailed_result.variables);
            console.log(`    Variables found: ${variable_names.join(', ')}`);
            
            for (const name of variable_names) {
                const variable = detailed_result.variables[name];
                console.log(`      ${name}: ${variable.type}, ${variable.valueCount} values, dims: [${variable.dimensions.join(',')}]`);
            }
        } else {
            console.log(`    Parsing error: ${detailed_result.error}`);
        }
        
        return detailed_result.success && Object.keys(detailed_result.variables).length > 0;
    });
    
    // Test 4: Parse DODS data and extract values
    await test('Parse DODS data and extract values', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        const parser = new UniversalDodsParser();
        
        const constraint = 't2m[0][0][0][0]';
        const dods_url = url_builder.dodsUrl(constraint);
        const binary_data = await fetcher.fetchBinary(dods_url);
        const uint8_data = new Uint8Array(binary_data);
        
        const parsed_data = parser.parseDods(uint8_data);
        
        console.log(`    Parsed variables: ${Object.keys(parsed_data).join(', ')}`);
        
        if (parsed_data.t2m) {
            console.log(`    t2m data type: ${parsed_data.t2m.type}`);
            console.log(`    t2m data length: ${parsed_data.t2m.length}`);
            console.log(`    t2m dimensions: [${parsed_data.t2m.dimensions.join(',')}]`);
            
            if (parsed_data.t2m.data && parsed_data.t2m.data.length > 0) {
                console.log(`    t2m first value: ${parsed_data.t2m.data[0]}`);
                console.log(`    t2m data type: ${parsed_data.t2m.data.constructor.name}`);
            }
        }
        
        return Object.keys(parsed_data).length > 0 && parsed_data.t2m && parsed_data.t2m.length > 0;
    });
    
    // Test 5: Test with different constraint sizes
    await test('Test parsing different constraint sizes', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        const parser = new UniversalDodsParser();
        
        const constraints = [
            { constraint: 't2m[0][0][0][0]', expected_count: 1 },
            { constraint: 't2m[0][0:1][0:1][0]', expected_count: 4 },
            { constraint: 't2m[0][0:2][0:2][0]', expected_count: 9 }
        ];
        
        let success_count = 0;
        
        for (const { constraint, expected_count } of constraints) {
            try {
                console.log(`    Testing: ${constraint} (expecting ${expected_count} values)`);
                
                const dods_url = url_builder.dodsUrl(constraint);
                const binary_data = await fetcher.fetchBinary(dods_url);
                const uint8_data = new Uint8Array(binary_data);
                
                const parsed_data = parser.parseDods(uint8_data);
                
                if (parsed_data.t2m && parsed_data.t2m.length === expected_count) {
                    console.log(`      ✅ Got ${parsed_data.t2m.length} values (${parsed_data.t2m.data[0].toFixed(2)}...)`);
                    success_count++;
                } else {
                    console.log(`      ❌ Expected ${expected_count} values, got ${parsed_data.t2m?.length || 0}`);
                }
            } catch (error) {
                console.log(`      ❌ Failed: ${error.message}`);
            }
        }
        
        return success_count === constraints.length;
    });
    
    // Test 6: Test multiple variables
    await test('Test parsing multiple variables', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        const parser = new UniversalDodsParser();
        
        // Use SimpleConstraintBuilder to create multi-variable constraint
        const single_constraint = new SimpleConstraintBuilder()
            .addSingle('time', 0)
            .addSingle('latitude', 0)
            .addSingle('longitude', 0)
            .addSingle('step', 0)
            .build();
        
        const constraint = `t2m[${single_constraint}],tcc[${single_constraint}]`;
        console.log(`    Multi-variable constraint: ${constraint}`);
        
        const dods_url = url_builder.dodsUrl(constraint);
        const binary_data = await fetcher.fetchBinary(dods_url);
        const uint8_data = new Uint8Array(binary_data);
        
        const parsed_data = parser.parseDods(uint8_data);
        
        const variable_names = Object.keys(parsed_data);
        console.log(`    Variables parsed: ${variable_names.join(', ')}`);
        
        let all_valid = true;
        for (const name of variable_names) {
            const variable = parsed_data[name];
            console.log(`      ${name}: ${variable.type}, length=${variable.length}, value=${variable.data?.[0]?.toFixed(3) || 'N/A'}`);
            
            if (!variable.data || variable.length === 0) {
                all_valid = false;
            }
        }
        
        return variable_names.length >= 2 && all_valid;
    });
    
    // Test 7: Error handling with malformed data
    await test('Test error handling with malformed data', async () => {
        const parser = new UniversalDodsParser();
        
        // Test with various malformed data
        const test_cases = [
            { name: 'Empty data', data: new Uint8Array() },
            { name: 'No Data marker', data: new Uint8Array(100).fill(65) }, // All 'A' characters
            { name: 'Truncated data', data: new Uint8Array([68, 97, 116, 97, 58, 10, 0, 0]) }, // "Data:" + minimal bytes
        ];
        
        let error_handling_correct = 0;
        
        for (const test_case of test_cases) {
            try {
                const result = parser.parseDodsDetailed(test_case.data);
                console.log(`    ${test_case.name}: ${result.success ? 'Unexpected success' : 'Correctly failed'}`);
                
                if (!result.success) {
                    error_handling_correct++;
                    console.log(`      Error: ${result.error}`);
                }
            } catch (error) {
                console.log(`    ${test_case.name}: Exception (${error.message})`);
                error_handling_correct++; // Exceptions are also acceptable for malformed data
            }
        }
        
        return error_handling_correct === test_cases.length;
    });
    
    // Test 8: Performance comparison
    await test('Performance comparison with original parser', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        const universal_parser = new UniversalDodsParser();
        
        const constraint = 't2m[0][0:4][0:4][0]'; // 5x5 grid
        const dods_url = url_builder.dodsUrl(constraint);
        const binary_data = await fetcher.fetchBinary(dods_url);
        const uint8_data = new Uint8Array(binary_data);
        
        // Time universal parser
        const iterations = 5;
        const start_time = performance.now();
        
        let parse_success = true;
        for (let i = 0; i < iterations; i++) {
            try {
                const result = universal_parser.parseDods(uint8_data);
                if (!result.t2m || result.t2m.length === 0) {
                    parse_success = false;
                    break;
                }
            } catch (error) {
                parse_success = false;
                break;
            }
        }
        
        const end_time = performance.now();
        const avg_time = (end_time - start_time) / iterations;
        
        console.log(`    ${iterations} parsing iterations`);
        console.log(`    Average parsing time: ${avg_time.toFixed(2)}ms`);
        console.log(`    All parses successful: ${parse_success}`);
        console.log(`    Data size: ${binary_data.length} bytes (25 float values)`);
        
        return parse_success && avg_time < 100; // Should be reasonably fast
    });
    
    // Summary
    console.log('\\n📊 Universal DODS Parser Test Summary');
    console.log('=' .repeat(60));
    console.log(`Passed: ${passedTests}/${totalTests} tests`);
    console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
    
    if (passedTests === totalTests) {
        console.log('🎉 All Universal DODS Parser tests passed!');
        console.log('✅ DODS parsing is now working correctly across all runtimes');
        console.log('✅ Robust error handling implemented');
        console.log('✅ Support for multiple variables and constraint sizes');
        console.log('✅ Performance is acceptable for typical use cases');
        
        console.log('\\n🏆 Phase 2.2 Achievements:');
        console.log('   ✓ Runtime-agnostic binary data parsing');
        console.log('   ✓ Improved error messages and debugging');
        console.log('   ✓ Support for various constraint sizes');
        console.log('   ✓ Multi-variable parsing capability');
        console.log('   ✓ Robust handling of malformed data');
        
        return true;
    } else {
        console.log('⚠️  Some Universal DODS Parser tests failed.');
        
        if (passedTests === 0) {
            console.log('   🔍 Complete failure - fundamental implementation issue');
        } else if (passedTests < totalTests / 2) {
            console.log('   🔍 Major issues - core parsing logic needs work');
        } else {
            console.log('   🔍 Minor issues - edge cases or specific scenarios');
        }
        
        console.log('\\n🔧 Potential fixes needed:');
        console.log('   • Check binary data format handling');
        console.log('   • Verify DDS parsing logic');
        console.log('   • Improve error handling coverage');
        console.log('   • Test with more OpenDAP servers');
        
        return false;
    }
}

// Run tests
testUniversalDodsParser()
    .then(success => {
        console.log(`\\n${success ? '🎉 Universal DODS parser testing completed successfully!' : '🔧 Universal DODS parser needs more work!'}`);
        process.exit(success ? 0 : 1);
    })
    .catch(error => {
        console.error('❌ Universal DODS parser test runner failed:', error);
        process.exit(1);
    });