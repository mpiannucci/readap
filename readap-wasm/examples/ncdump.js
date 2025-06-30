#!/usr/bin/env node
/**
 * ncdump.js - OpenDAP equivalent of ncdump for viewing dataset metadata and data
 * 
 * Usage:
 *   node ncdump.js <opendap_url> [options]
 *   bun ncdump.js <opendap_url> [options]  
 *   deno run --allow-net ncdump.js <opendap_url> [options]
 * 
 * Options:
 *   -h, --header     Show header (metadata) only
 *   -v, --variable   Show specific variable data
 *   -c, --constraint Add constraint (e.g., "time[0:5]")
 *   --help          Show this help
 * 
 * Examples:
 *   node ncdump.js https://server.com/data.nc --header
 *   bun ncdump.js https://server.com/data.nc -v temperature -c "time[0]"
 */

import init, { 
    ImmutableDataset, 
    SimpleConstraintBuilder,
    UniversalFetch 
} from '../pkg/readap_wasm.js';

function showUsage() {
    console.log(`
ncdump.js - OpenDAP dataset inspector

Usage: ncdump.js <opendap_url> [options]

Options:
  -h, --header      Show header (metadata) only
  -v, --variable    Show specific variable data  
  -c, --constraint  Add constraint (e.g., "time[0:5]")
  --help           Show this help

Examples:
  node ncdump.js https://server.com/data.nc --header
  bun ncdump.js https://server.com/data.nc -v temperature -c "time[0]"
  deno run --allow-net ncdump.js https://server.com/data.nc
`);
}

function detectRuntime() {
    if (typeof Bun !== 'undefined') return 'Bun';
    if (typeof Deno !== 'undefined') return 'Deno';
    if (typeof process !== 'undefined' && process.versions?.node) return 'Node.js';
    return 'Unknown';
}

async function parseArgs() {
    const args = typeof Deno !== 'undefined' ? Deno.args : process.argv.slice(2);
    
    if (args.length === 0 || args.includes('--help')) {
        showUsage();
        process.exit(0);
    }
    
    const url = args[0];
    const options = {
        headerOnly: args.includes('-h') || args.includes('--header'),
        variable: null,
        constraint: null
    };
    
    // Parse variable option
    const varIndex = args.findIndex(arg => arg === '-v' || arg === '--variable');
    if (varIndex !== -1 && varIndex + 1 < args.length) {
        options.variable = args[varIndex + 1];
    }
    
    // Parse constraint option
    const constIndex = args.findIndex(arg => arg === '-c' || arg === '--constraint');
    if (constIndex !== -1 && constIndex + 1 < args.length) {
        options.constraint = args[constIndex + 1];
    }
    
    return { url, options };
}

async function showHeader(dataset) {
    console.log('📋 Dataset Metadata');
    console.log('=' .repeat(50));
    
    // Show basic info
    console.log(`Base URL: ${dataset.baseUrl()}`);
    console.log(`DAS URL:  ${dataset.dasUrl()}`);
    console.log(`DDS URL:  ${dataset.ddsUrl()}`);
    console.log('');
    
    // Show variables
    const variables = dataset.getVariableNames();
    console.log(`Variables (${variables.length}):`);
    
    for (let i = 0; i < variables.length; i++) {
        const varName = variables[i];
        try {
            const info = JSON.parse(dataset.getVariableInfo(varName));
            console.log(`  ${varName}:`);
            console.log(`    Type: ${info.data_type}`);
            if (info.dimensions.length > 0) {
                console.log(`    Dimensions: [${info.dimensions.join(', ')}]`);
            }
            if (Object.keys(info.attributes).length > 0) {
                console.log(`    Attributes: ${Object.keys(info.attributes).length} items`);
            }
        } catch (error) {
            console.log(`  ${varName}: (info unavailable)`);
        }
        console.log('');
    }
}

async function showVariable(dataset, varName, constraint = null) {
    console.log(`📊 Variable Data: ${varName}`);
    console.log('=' .repeat(50));
    
    try {
        // Build constraint if provided
        let constraintStr = null;
        if (constraint) {
            constraintStr = `${varName}[${constraint}]`;
            console.log(`Constraint: ${constraintStr}`);
        } else {
            // Use minimal constraint to avoid huge downloads
            constraintStr = `${varName}[0:4]`; // First 5 elements max
            console.log(`Constraint: ${constraintStr} (limited to first 5 elements)`);
        }
        
        const data = await dataset.getVariable(varName, constraintStr);
        
        console.log(`Type: ${data.type}`);
        console.log(`Length: ${data.length}`);
        
        if (data.dimensions) {
            console.log(`Dimensions: [${data.dimensions.join(', ')}]`);
        }
        
        if (data.data) {
            const values = Array.from(data.data);
            console.log(`\\nData values (showing ${Math.min(values.length, 10)}):`);
            
            values.slice(0, 10).forEach((val, i) => {
                if (typeof val === 'number') {
                    console.log(`  [${i}] ${val.toFixed(6)}`);
                } else {
                    console.log(`  [${i}] ${val}`);
                }
            });
            
            if (values.length > 10) {
                console.log(`  ... (${values.length - 10} more values)`);
            }
            
            // Basic statistics for numeric data
            if (values.length > 0 && typeof values[0] === 'number') {
                const min = Math.min(...values);
                const max = Math.max(...values);
                const mean = values.reduce((a, b) => a + b, 0) / values.length;
                
                console.log(`\\nStatistics:`);
                console.log(`  Min:  ${min.toFixed(6)}`);
                console.log(`  Max:  ${max.toFixed(6)}`);
                console.log(`  Mean: ${mean.toFixed(6)}`);
            }
        }
        
    } catch (error) {
        console.error(`❌ Error reading variable '${varName}': ${error.message}`);
    }
}

async function showAllVariables(dataset) {
    console.log('📊 Dataset Overview');
    console.log('=' .repeat(50));
    
    const variables = dataset.getVariableNames();
    console.log(`Found ${variables.length} variables\\n`);
    
    // Show each variable with minimal data
    for (let i = 0; i < Math.min(variables.length, 5); i++) {
        const varName = variables[i];
        console.log(`${i + 1}. ${varName}:`);
        
        try {
            // Get just one data point to show the variable works
            const constraintStr = `${varName}[0]`;
            const data = await dataset.getVariable(varName, constraintStr);
            
            console.log(`   Type: ${data.type}, Length: ${data.length}`);
            if (data.data && data.data.length > 0) {
                const val = data.data[0];
                if (typeof val === 'number') {
                    console.log(`   Sample value: ${val.toFixed(6)}`);
                } else {
                    console.log(`   Sample value: ${val}`);
                }
            }
        } catch (error) {
            console.log(`   Error: ${error.message}`);
        }
        console.log('');
    }
    
    if (variables.length > 5) {
        console.log(`... and ${variables.length - 5} more variables`);
        console.log('\\nUse -v <variable_name> to see specific variable data');
    }
}

async function main() {
    try {
        await init();
        
        const { url, options } = await parseArgs();
        const runtime = detectRuntime();
        
        console.log(`🌐 ncdump.js running on ${runtime}`);
        console.log(`📡 Loading: ${url}`);
        console.log('');
        
        // Load dataset
        const dataset = await ImmutableDataset.fromURL(url);
        
        if (options.headerOnly) {
            await showHeader(dataset);
        } else if (options.variable) {
            await showVariable(dataset, options.variable, options.constraint);
        } else {
            // Show header and sample data
            await showHeader(dataset);
            await showAllVariables(dataset);
        }
        
    } catch (error) {
        console.error(`❌ Error: ${error.message}`);
        
        if (error.message.includes('network') || error.message.includes('fetch')) {
            console.error('\\n💡 Check that the URL is accessible and supports CORS');
        } else if (error.message.includes('parse') || error.message.includes('format')) {
            console.error('\\n💡 Check that the URL points to a valid OpenDAP endpoint');
        }
        
        process.exit(1);
    }
}

// Handle different runtime environments
if (typeof require !== 'undefined' || typeof import.meta !== 'undefined') {
    main().catch(error => {
        console.error('Fatal error:', error);
        process.exit(1);
    });
}