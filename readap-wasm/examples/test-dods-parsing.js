#!/usr/bin/env bun
// Test DODS binary data parsing across different runtimes

import init, { 
    UniversalFetch, 
    SimpleConstraintBuilder, 
    OpenDAPUrlBuilder, 
    OpenDAPDataset
} from '../pkg/readap_wasm.js';

const BASE_URL = 'https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap';

async function testDodsParsingIssues() {
    console.log('🔍 Testing DODS Binary Data Parsing Issues');
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
            console.log(`    Stack: ${error.stack}`);
        });
    }
    
    // Test 1: Fetch and examine raw DODS data
    await test('Fetch raw DODS data and examine structure', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        
        // Very simple constraint for minimal data
        const constraint = 't2m[0][0][0][0]';
        const dods_url = url_builder.dodsUrl(constraint);
        
        console.log(`    Fetching: ${dods_url}`);
        const binary_data = await fetcher.fetchBinary(dods_url);
        
        console.log(`    Binary data size: ${binary_data.length} bytes`);
        
        // Look for the "Data:" marker
        const text_portion = new TextDecoder().decode(binary_data.slice(0, Math.min(1000, binary_data.length)));
        console.log(`    Text portion contains "Data:": ${text_portion.includes('Data:')}`);
        
        if (text_portion.includes('Data:')) {
            const data_index = text_portion.indexOf('Data:');
            console.log(`    "Data:" found at position: ${data_index}`);
            console.log(`    Text before binary: ${text_portion.slice(0, data_index + 10)}`);
        }
        
        // Examine first 32 bytes in hex
        const hex_preview = Array.from(binary_data.slice(0, 32))
            .map(b => b.toString(16).padStart(2, '0'))
            .join(' ');
        console.log(`    First 32 bytes (hex): ${hex_preview}`);
        
        return binary_data.length > 0;
    });
    
    // Test 2: Try parsing with OpenDAPDataset
    await test('Parse DODS data with OpenDAPDataset', async () => {
        try {
            const dataset = await OpenDAPDataset.fromURL(BASE_URL);
            
            const fetcher = new UniversalFetch();
            const url_builder = new OpenDAPUrlBuilder(BASE_URL);
            const constraint = 't2m[0][0][0][0]';
            const dods_url = url_builder.dodsUrl(constraint);
            const binary_data = await fetcher.fetchBinary(dods_url);
            
            console.log(`    Attempting to parse ${binary_data.length} bytes of DODS data`);
            
            // Convert to Uint8Array for parsing
            const uint8_data = new Uint8Array(binary_data);
            const parsed_data = dataset.parseDODS(uint8_data);
            
            console.log(`    Parsing successful! Got object with keys: ${Object.keys(parsed_data)}`);
            
            if (parsed_data.t2m) {
                console.log(`    t2m data type: ${parsed_data.t2m.type}`);
                console.log(`    t2m data length: ${parsed_data.t2m.length}`);
            }
            
            return Object.keys(parsed_data).length > 0;
        } catch (error) {
            console.log(`    Parsing failed: ${error.message}`);
            return false;
        }
    });
    
    // Test 3: Try different constraint sizes
    await test('Test parsing with different constraint sizes', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        const dataset = await OpenDAPDataset.fromURL(BASE_URL);
        
        const constraints = [
            't2m[0][0][0][0]',           // Single point
            't2m[0][0:1][0:1][0]',       // 2x2 grid
            't2m[0][0:2][0:2][0]',       // 3x3 grid
        ];
        
        let success_count = 0;
        
        for (const constraint of constraints) {
            try {
                console.log(`    Testing constraint: ${constraint}`);
                const dods_url = url_builder.dodsUrl(constraint);
                const binary_data = await fetcher.fetchBinary(dods_url);
                
                const uint8_data = new Uint8Array(binary_data);
                const parsed_data = dataset.parseDODS(uint8_data);
                
                console.log(`      ✅ Size ${binary_data.length} bytes -> ${Object.keys(parsed_data).length} variables`);
                success_count++;
            } catch (error) {
                console.log(`      ❌ Failed: ${error.message}`);
            }
        }
        
        return success_count > 0;
    });
    
    // Test 4: Examine binary structure in detail
    await test('Examine DODS binary structure in detail', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        
        const constraint = 't2m[0][0][0][0]';
        const dods_url = url_builder.dodsUrl(constraint);
        const binary_data = await fetcher.fetchBinary(dods_url);
        
        // Find the "Data:" marker
        const full_text = new TextDecoder().decode(binary_data);
        const data_marker_index = full_text.indexOf('Data:');
        
        if (data_marker_index === -1) {
            console.log(`    ❌ No "Data:" marker found in response`);
            return false;
        }
        
        console.log(`    "Data:" marker found at position: ${data_marker_index}`);
        
        // Binary data starts after "Data:\\n"
        const binary_start = data_marker_index + 6; // "Data:" + "\\n" = 6 characters
        const binary_portion = binary_data.slice(binary_start);
        
        console.log(`    Binary portion length: ${binary_portion.length} bytes`);
        
        if (binary_portion.length >= 8) {
            // First 8 bytes should be the count (twice, as per OpenDAP format)
            const view = new DataView(binary_portion.buffer, binary_portion.byteOffset, binary_portion.byteLength);
            const count1 = view.getUint32(0, false); // big-endian
            const count2 = view.getUint32(4, false); // big-endian
            
            console.log(`    First count (big-endian): ${count1}`);
            console.log(`    Second count (big-endian): ${count2}`);
            console.log(`    Counts match: ${count1 === count2}`);
            
            if (binary_portion.length >= 12 && count1 === count2 && count1 > 0) {
                // Try to read the actual data value
                try {
                    const data_value = view.getFloat32(8, false); // big-endian float32
                    console.log(`    Data value (float32): ${data_value}`);
                    return true;
                } catch (error) {
                    console.log(`    Error reading data value: ${error.message}`);
                }
            }
        }
        
        return false;
    });
    
    // Test 5: Runtime-specific parsing behavior
    await test('Test runtime-specific parsing behavior', async () => {
        const fetcher = new UniversalFetch();
        const runtime_info = fetcher.getRuntimeInfo();
        console.log(`    Runtime: ${runtime_info}`);
        
        // Test basic binary parsing operations
        const test_data = new Uint8Array([0x00, 0x00, 0x00, 0x01, 0x40, 0x20, 0x00, 0x00]); // count=1, float=2.5
        const view = new DataView(test_data.buffer);
        
        const count = view.getUint32(0, false);
        const float_val = view.getFloat32(4, false);
        
        console.log(`    Test binary parsing - count: ${count}, float: ${float_val}`);
        console.log(`    Expected: count=1, float=2.5`);
        
        const parsing_correct = count === 1 && Math.abs(float_val - 2.5) < 0.001;
        console.log(`    Basic binary parsing: ${parsing_correct ? 'CORRECT' : 'INCORRECT'}`);
        
        return parsing_correct;
    });
    
    // Test 6: Compare against manual binary parsing
    await test('Compare WASM parsing vs manual parsing', async () => {
        const fetcher = new UniversalFetch();
        const url_builder = new OpenDAPUrlBuilder(BASE_URL);
        
        const constraint = 't2m[0][0][0][0]';
        const dods_url = url_builder.dodsUrl(constraint);
        const binary_data = await fetcher.fetchBinary(dods_url);
        
        // Manual parsing
        const full_text = new TextDecoder().decode(binary_data);
        const data_marker_index = full_text.indexOf('Data:');
        
        if (data_marker_index === -1) {
            console.log(`    No Data: marker found`);
            return false;
        }
        
        const binary_start = data_marker_index + 6;
        const binary_portion = binary_data.slice(binary_start);
        
        let manual_result = null;
        if (binary_portion.length >= 12) {
            const view = new DataView(binary_portion.buffer, binary_portion.byteOffset, binary_portion.byteLength);
            const count1 = view.getUint32(0, false);
            const count2 = view.getUint32(4, false);
            
            if (count1 === count2 && count1 === 1) {
                manual_result = view.getFloat32(8, false);
                console.log(`    Manual parsing result: ${manual_result}`);
            }
        }
        
        // WASM parsing
        let wasm_result = null;
        try {
            const dataset = await OpenDAPDataset.fromURL(BASE_URL);
            const uint8_data = new Uint8Array(binary_data);
            const parsed_data = dataset.parseDODS(uint8_data);
            
            if (parsed_data.t2m && parsed_data.t2m.data && parsed_data.t2m.data.length > 0) {
                wasm_result = parsed_data.t2m.data[0];
                console.log(`    WASM parsing result: ${wasm_result}`);
            }
        } catch (error) {
            console.log(`    WASM parsing failed: ${error.message}`);
        }
        
        if (manual_result !== null && wasm_result !== null) {
            const values_match = Math.abs(manual_result - wasm_result) < 0.001;
            console.log(`    Values match: ${values_match}`);
            return values_match;
        } else if (manual_result !== null) {
            console.log(`    Manual parsing succeeded, WASM failed`);
            return false;
        } else {
            console.log(`    Both parsing methods failed`);
            return false;
        }
    });
    
    // Summary
    console.log('\\n📊 DODS Parsing Test Summary');
    console.log('=' .repeat(60));
    console.log(`Passed: ${passedTests}/${totalTests} tests`);
    console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
    
    if (passedTests === totalTests) {
        console.log('🎉 All DODS parsing tests passed!');
        console.log('✅ DODS binary parsing is working correctly');
        return true;
    } else {
        console.log('⚠️  Some DODS parsing tests failed. Issues identified:');
        
        if (passedTests === 0) {
            console.log('   🔍 All tests failed - fundamental parsing issue');
            console.log('   💡 Check binary data format and nom parser compatibility');
        } else if (passedTests < totalTests / 2) {
            console.log('   🔍 Major parsing issues detected');
            console.log('   💡 Check runtime-specific binary handling differences');
        } else {
            console.log('   🔍 Minor parsing issues detected');
            console.log('   💡 Some edge cases may need handling');
        }
        
        console.log('\\n🔧 Next steps for Phase 2.2:');
        console.log('   1. Implement robust binary data parsing');
        console.log('   2. Add runtime-specific compatibility layers');
        console.log('   3. Improve error handling for malformed data');
        console.log('   4. Add comprehensive binary format validation');
        
        return false;
    }
}

// Run tests
testDodsParsingIssues()
    .then(success => {
        console.log(`\\n${success ? '🎉 DODS parsing analysis completed!' : '🔍 DODS parsing issues identified!'}`);
        process.exit(success ? 0 : 1);
    })
    .catch(error => {
        console.error('❌ DODS parsing test runner failed:', error);
        process.exit(1);
    });