#!/usr/bin/env node
// Node.js-specific test for constraint builders
// Uses fs to read WASM file directly

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import initWasm, { SimpleConstraintBuilder, StringConstraintBuilder, OpenDAPUrlBuilder } from '../pkg/readap_wasm.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function testConstraintBuilders() {
    console.log('🧪 Testing Universal Constraint Builders (Node.js)');
    console.log('=' .repeat(50));
    
    // Load WASM file directly for Node.js
    const wasmPath = join(__dirname, '../pkg/readap_wasm_bg.wasm');
    const wasmBytes = readFileSync(wasmPath);
    await initWasm(wasmBytes);
    
    let passedTests = 0;
    let totalTests = 0;
    
    function test(description, testFn) {
        totalTests++;
        try {
            const result = testFn();
            if (result) {
                console.log(`✅ ${description}`);
                passedTests++;
            } else {
                console.log(`❌ ${description} - Test returned false`);
            }
        } catch (error) {
            console.log(`❌ ${description} - Error: ${error.message}`);
        }
    }
    
    // Run the same tests as the universal version
    console.log('\n🔧 Testing SimpleConstraintBuilder (Node.js)');
    
    test('Create SimpleConstraintBuilder', () => {
        const builder = new SimpleConstraintBuilder();
        return builder !== null;
    });
    
    test('Single index constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const result = builder.addSingle('time', 5);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.includes('time') && constraint.includes('[5]');
    });
    
    test('Range constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const result = builder.addRange('latitude', 10, 20);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.includes('latitude') && constraint.includes('[10:20]');
    });
    
    test('Method chaining', () => {
        const builder = new SimpleConstraintBuilder();
        const result = builder
            .addSingle('time', 0)
            .addRange('latitude', 10, 20)
            .addSingle('step', 1);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.includes('time') && constraint.includes('latitude') && constraint.includes('step');
    });
    
    test('Multiple indices constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const indices = new Uint32Array([0, 5, 10, 15]);
        const result = builder.addMultiple('pressure', indices);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.includes('pressure') && constraint.includes('[');
    });
    
    console.log('\n📝 Testing StringConstraintBuilder (Node.js)');
    
    test('Create StringConstraintBuilder', () => {
        const builder = new StringConstraintBuilder();
        return builder !== null;
    });
    
    test('Add constraints by string', () => {
        const builder = new StringConstraintBuilder();
        const result = builder
            .addConstraint('time[0]')
            .addConstraint('lat[10:20]')
            .addVariable('temperature');
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint === 'time[0],lat[10:20],temperature';
    });
    
    console.log('\n🔗 Testing URL Builder Integration (Node.js)');
    
    test('URL builder with simple constraints', () => {
        const urlBuilder = new OpenDAPUrlBuilder('http://example.com/data');
        const constraint = new SimpleConstraintBuilder()
            .addSingle('time', 0)
            .addRange('lat', 10, 20)
            .build();
        const url = urlBuilder.dodsUrl(constraint);
        console.log(`    URL: ${url}`);
        return url.includes('time') && url.includes('lat') && url.includes('.dods?');
    });
    
    // Summary
    console.log('\n📊 Test Summary (Node.js)');
    console.log('=' .repeat(50));
    console.log(`Passed: ${passedTests}/${totalTests} tests`);
    console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
    
    if (passedTests === totalTests) {
        console.log('🎉 All Node.js tests passed! Constraint builders work in Node.js.');
        return true;
    } else {
        console.log('⚠️  Some Node.js tests failed. Check the implementation.');
        return false;
    }
}

// Run tests
testConstraintBuilders()
    .then(success => {
        process.exit(success ? 0 : 1);
    })
    .catch(error => {
        console.error('❌ Node.js test runner failed:', error);
        process.exit(1);
    });