#!/usr/bin/env bun
// Comprehensive test for the new constraint builders
// Works in Browser, Node.js, Bun, and Deno

import init, { SimpleConstraintBuilder, StringConstraintBuilder, OpenDAPUrlBuilder } from '../pkg/readap_wasm.js';

async function testConstraintBuilders() {
    console.log('🧪 Testing Universal Constraint Builders');
    console.log('=' .repeat(50));
    
    await init();
    
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
    
    // Test SimpleConstraintBuilder
    console.log('\n🔧 Testing SimpleConstraintBuilder');
    
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
    
    test('Stride constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const result = builder.addStride('longitude', 0, 2, 10);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.includes('longitude') && constraint.includes('[0:2:10]');
    });
    
    test('Multiple indices constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const indices = new Uint32Array([0, 5, 10, 15]);
        const result = builder.addMultiple('pressure', indices);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.includes('pressure') && constraint.includes('[');
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
    
    test('Value-based single constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const result = builder.addValueSingle('temperature', 25.5);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.length > 0; // Value constraints need coordinate resolution
    });
    
    test('Value-based range constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const result = builder.addValueRange('time', 0.0, 24.0);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.length > 0;
    });
    
    test('String-based constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const result = builder.addValueString('time', '2023-01-15T12:00:00Z');
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.length > 0;
    });
    
    test('Multiple values constraint', () => {
        const builder = new SimpleConstraintBuilder();
        const values = new Float64Array([1.0, 2.5, 5.0]);
        const result = builder.addValueMultiple('depth', values);
        const constraint = result.build();
        console.log(`    Generated: "${constraint}"`);
        return constraint.length > 0;
    });
    
    test('Builder cloning', () => {
        const builder1 = new SimpleConstraintBuilder().addSingle('time', 0);
        const builder2 = builder1.clone().addRange('lat', 10, 20);
        const constraint1 = builder1.build();
        const constraint2 = builder2.build();
        console.log(`    Original: "${constraint1}"`);
        console.log(`    Cloned:   "${constraint2}"`);
        return constraint1 !== constraint2 && constraint2.includes(constraint1);
    });
    
    test('Builder reset', () => {
        const builder = new SimpleConstraintBuilder()
            .addSingle('time', 0)
            .addRange('lat', 10, 20);
        const beforeReset = builder.build();
        const afterReset = builder.reset().build();
        console.log(`    Before reset: "${beforeReset}"`);
        console.log(`    After reset:  "${afterReset}"`);
        return beforeReset.length > 0 && afterReset.length === 0;
    });
    
    // Test StringConstraintBuilder
    console.log('\n📝 Testing StringConstraintBuilder');
    
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
    
    test('String builder count', () => {
        const builder = new StringConstraintBuilder()
            .addConstraint('time[0]')
            .addConstraint('lat[10:20]');
        const count = builder.getCount();
        console.log(`    Count: ${count}`);
        return count === 2;
    });
    
    test('String builder reset', () => {
        const builder = new StringConstraintBuilder()
            .addConstraint('time[0]')
            .addConstraint('lat[10:20]');
        const beforeCount = builder.getCount();
        const afterCount = builder.reset().getCount();
        console.log(`    Before reset count: ${beforeCount}`);
        console.log(`    After reset count:  ${afterCount}`);
        return beforeCount === 2 && afterCount === 0;
    });
    
    // Test integration with URL builder
    console.log('\n🔗 Testing URL Builder Integration');
    
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
    
    test('URL builder with string constraints', () => {
        const urlBuilder = new OpenDAPUrlBuilder('http://example.com/data');
        const constraint = new StringConstraintBuilder()
            .addConstraint('temperature[0][10:20][0:5]')
            .addVariable('pressure')
            .build();
        const url = urlBuilder.dodsUrl(constraint);
        console.log(`    URL: ${url}`);
        return url.includes('temperature') && url.includes('pressure');
    });
    
    // Test error handling and edge cases
    console.log('\n🛡️ Testing Error Handling');
    
    test('Empty constraint builder', () => {
        const builder = new SimpleConstraintBuilder();
        const constraint = builder.build();
        console.log(`    Empty constraint: "${constraint}"`);
        return constraint === '';
    });
    
    test('Zero-length arrays', () => {
        const builder = new SimpleConstraintBuilder();
        const emptyIndices = new Uint32Array([]);
        const result = builder.addMultiple('test', emptyIndices);
        const constraint = result.build();
        console.log(`    Empty array constraint: "${constraint}"`);
        return constraint.length >= 0; // Should not crash
    });
    
    // Test memory management
    console.log('\n💾 Testing Memory Management');
    
    test('Multiple builder instances', () => {
        const builders = [];
        for (let i = 0; i < 10; i++) {
            builders.push(new SimpleConstraintBuilder().addSingle('test', i));
        }
        const constraints = builders.map(b => b.build());
        console.log(`    Created ${builders.length} builders`);
        return constraints.length === 10;
    });
    
    test('Cleanup builders', () => {
        for (let i = 0; i < 10; i++) {
            const builder = new SimpleConstraintBuilder().addSingle('cleanup', i);
            builder.free(); // Explicit cleanup
        }
        console.log(`    Cleaned up 10 builders`);
        return true;
    });
    
    // Summary
    console.log('\n📊 Test Summary');
    console.log('=' .repeat(50));
    console.log(`Passed: ${passedTests}/${totalTests} tests`);
    console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);
    
    if (passedTests === totalTests) {
        console.log('🎉 All tests passed! Constraint builders are working correctly.');
        return true;
    } else {
        console.log('⚠️  Some tests failed. Check the implementation.');
        return false;
    }
}

// Run tests
testConstraintBuilders()
    .then(success => {
        process.exit(success ? 0 : 1);
    })
    .catch(error => {
        console.error('❌ Test runner failed:', error);
        process.exit(1);
    });