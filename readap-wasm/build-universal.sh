#!/bin/bash

# Universal build script for readap-wasm

set -e  # Exit on error

echo "🚀 Building universal readap-wasm package..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg

# Build for bundler target (most compatible)
echo "📦 Building WASM module..."
wasm-pack build --target bundler --out-dir pkg

# Create a universal wrapper that works everywhere
echo "🔧 Creating universal wrapper..."

cat > pkg/universal.js << 'EOF'
// Universal wrapper for readap-wasm that works in Node, Deno, Bun, and browsers

let wasm;
let wasmModule;

// Feature detection
const isNode = typeof process !== 'undefined' && process.versions && process.versions.node;
const isDeno = typeof Deno !== 'undefined';
const isBun = typeof Bun !== 'undefined';
const isBrowser = typeof window !== 'undefined' && typeof window.document !== 'undefined';

async function loadWasm() {
    if (wasm) return wasm;

    try {
        if (isNode && !isBun) {
            // Node.js
            const fs = require('fs');
            const path = require('path');
            const wasmPath = path.join(__dirname, 'readap_wasm_bg.wasm');
            const wasmBuffer = fs.readFileSync(wasmPath);
            wasmModule = await WebAssembly.instantiate(wasmBuffer);
            wasm = require('./readap_wasm.js');
            wasm.__wbg_set_wasm(wasmModule.instance.exports);
        } else if (isDeno) {
            // Deno
            const wasmPath = new URL('./readap_wasm_bg.wasm', import.meta.url);
            const wasmBuffer = await Deno.readFile(wasmPath);
            wasmModule = await WebAssembly.instantiate(wasmBuffer);
            wasm = await import('./readap_wasm.js');
            wasm.__wbg_set_wasm(wasmModule.instance.exports);
        } else if (isBun) {
            // Bun
            const wasmPath = new URL('./readap_wasm_bg.wasm', import.meta.url);
            const wasmBuffer = await Bun.file(wasmPath).arrayBuffer();
            wasmModule = await WebAssembly.instantiate(wasmBuffer);
            wasm = await import('./readap_wasm.js');
            wasm.__wbg_set_wasm(wasmModule.instance.exports);
        } else if (isBrowser) {
            // Browser
            wasm = await import('./readap_wasm.js');
            await wasm.default();
        } else {
            // Fallback for bundlers
            wasm = await import('./readap_wasm.js');
            if (wasm.default) {
                await wasm.default();
            }
        }
    } catch (error) {
        console.error('Failed to load WASM module:', error);
        throw error;
    }

    return wasm;
}

// Export everything
module.exports = loadWasm;
module.exports.loadWasm = loadWasm;

// For ES modules
export default loadWasm;
export { loadWasm };
EOF

# Update package.json to support all environments
echo "📝 Updating package.json..."

cat > pkg/package.json << 'EOF'
{
  "name": "readap-wasm",
  "version": "0.1.0",
  "description": "Universal WebAssembly bindings for the readap OpenDAP parser",
  "main": "./readap_wasm.js",
  "module": "./readap_wasm.js",
  "browser": "./readap_wasm.js",
  "types": "./readap_wasm.d.ts",
  "exports": {
    ".": {
      "types": "./readap_wasm.d.ts",
      "browser": "./readap_wasm.js",
      "node": {
        "import": "./universal.js",
        "require": "./universal.js"
      },
      "deno": "./universal.js",
      "bun": "./universal.js",
      "default": "./readap_wasm.js"
    },
    "./universal": {
      "types": "./readap_wasm.d.ts",
      "import": "./universal.js",
      "require": "./universal.js"
    },
    "./readap_wasm_bg.wasm": "./readap_wasm_bg.wasm",
    "./readap_wasm_bg.js": "./readap_wasm_bg.js"
  },
  "files": [
    "readap_wasm_bg.wasm",
    "readap_wasm_bg.js",
    "readap_wasm.js",
    "readap_wasm.d.ts",
    "universal.js"
  ],
  "sideEffects": false,
  "keywords": [
    "opendap",
    "dap2",
    "wasm",
    "webassembly",
    "universal"
  ],
  "engines": {
    "node": ">=14.0.0"
  },
  "author": "Matthew Iannucci <mpiannucci@gmail.com>",
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/mpiannucci/readap"
  }
}
EOF

echo "✅ Universal build complete! Package created in pkg/"
echo ""
echo "📖 Usage examples:"
echo ""
echo "Node.js (CommonJS):"
echo "  const loadWasm = require('readap-wasm/universal');"
echo "  const wasm = await loadWasm();"
echo ""
echo "Node.js/Deno/Bun (ES Modules):"
echo "  import loadWasm from 'readap-wasm/universal';"
echo "  const wasm = await loadWasm();"
echo ""
echo "Browser (with bundler):"
echo "  import * as wasm from 'readap-wasm';"
echo "  await wasm.default();"