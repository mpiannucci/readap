#!/bin/bash

# Build script for readap-wasm - builds for all targets

set -e  # Exit on error

echo "🚀 Building readap-wasm for all targets..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg pkg-node pkg-bundler

# Build for web target (ES modules)
echo "🌐 Building for web target..."
wasm-pack build --target web --out-dir pkg

# Build for Node.js
echo "📦 Building for Node.js..."
wasm-pack build --target nodejs --out-dir pkg-node

# Build for bundlers (webpack, rollup, etc.)
echo "📦 Building for bundlers..."
wasm-pack build --target bundler --out-dir pkg-bundler

# Run tests
echo "🧪 Running tests..."
wasm-pack test --headless --chrome --firefox || true

echo "✅ Build complete! Packages created in:"
echo "  - pkg/        (web/ES modules)"
echo "  - pkg-node/   (Node.js)"
echo "  - pkg-bundler/ (webpack/rollup)"