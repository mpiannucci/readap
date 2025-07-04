#!/bin/bash

# Build script for readap-wasm

echo "Building readap-wasm..."

# Clean previous builds
rm -rf pkg

# Build for web target (ES modules)
echo "Building for web target..."
wasm-pack build --target web

# Optional: Build for other targets
# echo "Building for Node.js..."
# wasm-pack build --target nodejs --out-dir pkg-node

# echo "Building for bundlers..."
# wasm-pack build --target bundler --out-dir pkg-bundler

echo "Build complete!"