#!/usr/bin/env bun
// Simple local development server for the web viewer
// Run with: bun run serve.js

import { serve } from "bun";
import { readFileSync, existsSync } from "fs";
import { join, extname } from "path";

const PORT = 8080;
const ROOT_DIR = import.meta.dirname;

// MIME types for different file extensions
const MIME_TYPES = {
    '.html': 'text/html',
    '.js': 'application/javascript',
    '.wasm': 'application/wasm',
    '.css': 'text/css',
    '.json': 'application/json',
    '.md': 'text/markdown',
    '.ts': 'application/typescript'
};

function getMimeType(filePath) {
    const ext = extname(filePath).toLowerCase();
    return MIME_TYPES[ext] || 'text/plain';
}

function serveFile(filePath) {
    try {
        if (!existsSync(filePath)) {
            return new Response("File not found", { status: 404 });
        }

        const content = readFileSync(filePath);
        const mimeType = getMimeType(filePath);
        
        const headers = {
            'Content-Type': mimeType,
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
            'Access-Control-Allow-Headers': 'Content-Type, Authorization'
        };

        // Add WASM-specific headers
        if (mimeType === 'application/wasm') {
            headers['Cross-Origin-Embedder-Policy'] = 'require-corp';
            headers['Cross-Origin-Opener-Policy'] = 'same-origin';
        }

        return new Response(content, { headers });
    } catch (error) {
        return new Response(`Error reading file: ${error.message}`, { status: 500 });
    }
}

const server = serve({
    port: PORT,
    fetch(req) {
        const url = new URL(req.url);
        let pathname = url.pathname;
        
        // Handle CORS preflight
        if (req.method === 'OPTIONS') {
            return new Response(null, {
                status: 200,
                headers: {
                    'Access-Control-Allow-Origin': '*',
                    'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
                    'Access-Control-Allow-Headers': 'Content-Type, Authorization'
                }
            });
        }
        
        // Default to index.html for root path
        if (pathname === '/') {
            pathname = '/web-viewer.html';
        }
        
        // Remove leading slash and resolve file path
        const relativePath = pathname.slice(1);
        
        // Handle pkg directory access
        if (relativePath.startsWith('pkg/')) {
            const pkgPath = join(ROOT_DIR, '..', relativePath);
            return serveFile(pkgPath);
        }
        
        // Handle examples directory
        const filePath = join(ROOT_DIR, relativePath);
        return serveFile(filePath);
    },
});

console.log(`🌐 Local development server running at http://localhost:${PORT}`);
console.log(`📁 Serving files from: ${ROOT_DIR}`);
console.log(`🎯 Open http://localhost:${PORT} to view the web viewer`);
console.log('');
console.log('Available endpoints:');
console.log(`  • http://localhost:${PORT}/web-viewer.html - Interactive web viewer`);
console.log(`  • http://localhost:${PORT}/fetch-example.js - Original fetch example`);
console.log(`  • http://localhost:${PORT}/pkg/ - WASM package files`);
console.log('');
console.log('Press Ctrl+C to stop the server');