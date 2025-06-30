#!/usr/bin/env node

/**
 * metadata-inspector.js - Simple OpenDAP metadata inspection tool
 * 
 * Usage: node metadata-inspector.js <URL>
 *        bun metadata-inspector.js <URL>
 * 
 * Fetches and displays metadata information from OpenDAP datasets including:
 * - Available variables with their types and dimensions
 * - Coordinate variables and their sizes
 * - Basic dataset structure
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import init, { ImmutableDataset, universalFetchText } from '../pkg/readap_wasm.js';

// Runtime detection
const isNode = typeof process !== 'undefined' && process.versions && process.versions.node;
const isBun = typeof Bun !== 'undefined';

/**
 * Initialize WASM with Node.js compatibility
 * Node.js cannot fetch file:// URLs, so we need to read the WASM file manually
 */
async function initializeWasm() {
    if (!isNode) {
        // For non-Node.js environments (Bun, browser), use default initialization
        await init();
        return;
    }
    
    try {
        // For Node.js, manually load the WASM file
        const scriptDir = dirname(fileURLToPath(import.meta.url));
        const wasmPath = join(scriptDir, '..', 'pkg', 'readap_wasm_bg.wasm');
        const wasmBytes = readFileSync(wasmPath);
        await init(wasmBytes);
    } catch (error) {
        console.error('Failed to load WASM file. Make sure you are running from the examples directory.');
        throw error;
    }
}

async function main() {
    // Check command line arguments
    if (process.argv.length < 3) {
        console.error('Usage: node metadata-inspector.js <URL>');
        console.error('       bun metadata-inspector.js <URL>');
        console.error('');
        console.error('Example: bun metadata-inspector.js https://example.com/data.nc');
        process.exit(1);
    }

    let url = process.argv[2];

    // Handle HTTP -> HTTPS redirect for earthmover.io URLs
    if (url.startsWith('http://compute.earthmover.io')) {
        url = url.replace('http://', 'https://');
        console.log(`Redirecting to HTTPS: ${url}`);
    }

    try {
        // Initialize WebAssembly with Node.js compatibility
        await initializeWasm();
        
        console.log(`Runtime: ${isBun ? 'Bun' : isNode ? 'Node.js' : 'Unknown'}`);
        console.log(`Dataset URL: ${url}`);
        
        console.log('');

        // Load dataset with metadata
        console.log('Loading dataset metadata...');
        const dataset = await ImmutableDataset.fromURL(url);
        console.log('✓ Dataset metadata loaded successfully');
        console.log('');

        // Get DDS content and parse it directly (workaround for library bug)
        console.log('Fetching DDS to extract dimension information...');
        const ddsContent = await universalFetchText(dataset.ddsUrl());
        const { dimensions, variables } = parseDDS(ddsContent);
        
        // Get basic variable info from the library (for data types, etc.)
        const variablesInfoJson = dataset.getVariablesInfo();
        const basicVarInfo = JSON.parse(variablesInfoJson);
        
        // Merge parsed DDS info with basic variable info
        for (const [varName, varData] of Object.entries(variables)) {
            if (basicVarInfo[varName]) {
                basicVarInfo[varName].dimensions = varData.dimensions;
                basicVarInfo[varName].shape = varData.shape;
            }
        }
        
        const variablesInfo = basicVarInfo;
        const coordinateVars = new Set(Object.keys(variables).filter(name => 
            variables[name].isCoordinate
        ));

        // Display dimensions
        console.log('=== DIMENSIONS ===');
        if (dimensions.size > 0) {
            for (const [name, size] of dimensions) {
                console.log(`  ${name}: ${size}`);
            }
        } else {
            console.log('  No dimensions found');
        }
        console.log('');

        // Display coordinate variables
        console.log('=== COORDINATE VARIABLES ===');
        if (coordinateVars.size > 0) {
            for (const varName of coordinateVars) {
                const varInfo = variablesInfo[varName];
                const dimInfo = varInfo.dimensions?.[0];
                const sizeStr = dimInfo ? `[${dimInfo.size}]` : '[scalar]';
                console.log(`  ${varName} (${varInfo.data_type}${sizeStr})`);
                
                // Show attributes if available
                if (varInfo.attributes) {
                    for (const [attrName, attrValue] of Object.entries(varInfo.attributes)) {
                        if (attrName === 'units' || attrName === 'standard_name' || attrName === 'long_name') {
                            console.log(`    ${attrName}: ${attrValue}`);
                        }
                    }
                }
            }
        } else {
            console.log('  No coordinate variables found');
        }
        console.log('');

        // Display data variables
        console.log('=== DATA VARIABLES ===');
        const dataVars = Object.entries(variablesInfo).filter(([varName]) => !coordinateVars.has(varName));
        
        if (dataVars.length > 0) {
            for (const [varName, varInfo] of dataVars) {
                const dimNames = varInfo.dimensions ? varInfo.dimensions.map(d => `${d.name}[${d.size}]`).join(', ') : 'scalar';
                const totalSize = calculateTotalSize(varInfo);
                
                console.log(`  ${varName} (${varInfo.data_type})`);
                console.log(`    Dimensions: ${dimNames}`);
                console.log(`    Total elements: ${totalSize.toLocaleString()}`);
                
                // Show key attributes
                if (varInfo.attributes) {
                    for (const [attrName, attrValue] of Object.entries(varInfo.attributes)) {
                        if (attrName === 'units' || attrName === 'standard_name' || attrName === 'long_name') {
                            console.log(`    ${attrName}: ${attrValue}`);
                        }
                    }
                }
                console.log('');
            }
        } else {
            console.log('  No data variables found');
        }

        // Summary statistics
        console.log('=== SUMMARY ===');
        console.log(`  Total variables: ${Object.keys(variablesInfo).length}`);
        console.log(`  Coordinate variables: ${coordinateVars.size}`);
        console.log(`  Data variables: ${dataVars.length}`);
        console.log(`  Dimensions: ${dimensions.size}`);
        
        // Identify small variables suitable for testing
        console.log('');
        console.log('=== VARIABLES SUITABLE FOR DATA TESTING ===');
        const smallVars = Object.entries(variablesInfo).filter(([_, varInfo]) => {
            const size = calculateTotalSize(varInfo);
            return size <= 1000; // Small enough to fetch quickly
        });
        
        if (smallVars.length > 0) {
            console.log('These variables are small enough to fetch for testing:');
            for (const [varName, varInfo] of smallVars) {
                const size = calculateTotalSize(varInfo);
                console.log(`  ${varName}: ${size.toLocaleString()} elements`);
            }
        } else {
            console.log('No small variables found - all variables are large');
        }

    } catch (error) {
        console.error(`Error: ${error.message}`);
        if (error.stack) {
            console.error(`Stack: ${error.stack}`);
        }
        process.exit(1);
    }
}

/**
 * Parse DDS content to extract dimensions and variable information
 * This is a workaround for the library's dimension parsing bug
 */
function parseDDS(ddsContent) {
    const dimensions = new Map();
    const variables = {};
    
    const lines = ddsContent.split('\n');
    
    for (const line of lines) {
        const trimmed = line.trim();
        
        // Parse coordinate variables: Float64 longitude[longitude = 1440];
        const coordMatch = trimmed.match(/^(Float\d+|Int\d+)\s+(\w+)\[(\w+)\s*=\s*(\d+)\];$/);
        if (coordMatch) {
            const [, dataType, varName, dimName, size] = coordMatch;
            const dimSize = parseInt(size);
            
            dimensions.set(dimName, dimSize);
            variables[varName] = {
                isCoordinate: true,
                dimensions: [{ name: dimName, size: dimSize }],
                shape: [dimSize]
            };
            continue;
        }
        
        // Parse grid array declarations: Float32 gust[longitude = 1440][latitude = 721][time = 1138][step = 209];
        const gridMatch = trimmed.match(/^(Float\d+|Int\d+)\s+(\w+)(\[.+\]);$/);
        if (gridMatch) {
            const [, dataType, varName, dimString] = gridMatch;
            
            // Extract all dimensions from [longitude = 1440][latitude = 721]...
            const dimMatches = [...dimString.matchAll(/\[(\w+)\s*=\s*(\d+)\]/g)];
            const varDimensions = [];
            const shape = [];
            
            for (const [, dimName, size] of dimMatches) {
                const dimSize = parseInt(size);
                dimensions.set(dimName, dimSize);
                varDimensions.push({ name: dimName, size: dimSize });
                shape.push(dimSize);
            }
            
            variables[varName] = {
                isCoordinate: false,
                dimensions: varDimensions,
                shape: shape
            };
        }
    }
    
    return { dimensions, variables };
}

/**
 * Calculate total number of elements in a variable
 */
function calculateTotalSize(varInfo) {
    if (!varInfo.dimensions || varInfo.dimensions.length === 0) {
        return 1; // Scalar variable
    }
    
    return varInfo.dimensions.reduce((total, dim) => total * (dim.size || 1), 1);
}

// Run the main function
main().catch(error => {
    console.error(`Fatal error: ${error.message}`);
    process.exit(1);
});