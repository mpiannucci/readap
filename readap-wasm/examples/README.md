# ReadAP WASM Examples

This directory contains examples demonstrating different ways to use the readap-wasm library for OpenDAP data access.

## Examples Overview

### 1. 📄 `fetch-example.js` - Basic WASM Usage
The original comprehensive example showing all library features.

### 2. 🏃 `bun-example.js` - Server-Side Analysis
A complete Bun script for meteorological data analysis with performance testing.

### 3. 🌐 `web-viewer.html` - Interactive Web Viewer
A web-based grid data visualizer with interactive controls.

### 4. 🛠️ `serve.js` - Development Server
A local development server to handle CORS issues.

## Quick Start

### Running the Bun Example
```bash
# Navigate to examples directory
cd readap-wasm/examples

# Run the Bun script
bun run bun-example.js
```

### Running the Web Viewer

#### Option 1: Use the Development Server (Recommended)
```bash
# Navigate to examples directory
cd readap-wasm/examples

# Start the local server
bun run serve.js

# Open your browser to http://localhost:8080
```

#### Option 2: Direct File Access
Simply open `web-viewer.html` in your browser, but you may encounter CORS issues.

### Running the Basic Example
```bash
# Navigate to examples directory
cd readap-wasm/examples

# Run with Node.js or Bun
node fetch-example.js
# or
bun run fetch-example.js
```

## CORS Issues and Solutions

When running the web viewer, you might encounter CORS (Cross-Origin Resource Sharing) errors. This is because browsers block loading WASM files from the local file system for security reasons.

### Solutions:

1. **Use the Development Server** (Recommended)
   ```bash
   bun run serve.js
   ```
   Then visit `http://localhost:8080`

2. **Chrome with Disabled Security** (For testing only)
   ```bash
   chrome --disable-web-security --user-data-dir=/tmp/chrome_dev
   ```

3. **Firefox Local File Access**
   - Type `about:config` in Firefox
   - Set `security.fileuri.strict_origin_policy` to `false`
   - ⚠️ Remember to reset this after testing

4. **Use a Different Web Server**
   ```bash
   # Python 3
   python -m http.server 8080
   
   # Python 2
   python -m SimpleHTTPServer 8080
   
   # Node.js (if you have http-server)
   npx http-server -p 8080
   ```

## Example Features

### Bun Script Features
- 📊 Comprehensive meteorological data analysis
- ⚡ Performance benchmarking
- 🎯 Multiple selection strategies (index-based, value-based, chained)
- 📈 Statistical analysis with min/max/average calculations
- 🔄 Multi-variable batch processing
- 💾 Memory-efficient data handling

### Known Issues with Bun

The WASM bindings currently have compatibility issues when running in Bun:

1. **Recursive object error** - Methods that take `&mut self` cause aliasing errors
2. **Constraint builder** - JavaScript object parsing fails with null pointer errors
3. **DODS parsing** - Returns "InvalidData" error

**Workaround**: Use `working-example.js` which demonstrates manual URL building and fetch operations that work correctly in Bun.

### Web Viewer Features
- 🎨 Interactive grid visualization with color coding
- 📱 Responsive design for different screen sizes
- 🎛️ Real-time controls for data selection
- 📊 Live statistical analysis display
- 🔄 Dynamic variable loading
- ❌ Comprehensive error handling

### Common Features Across All Examples
- 🌊 xarray-style data selection (`isel` and `sel`)
- 🎯 Nearest neighbor coordinate lookup
- 📡 Automatic metadata fetching
- 🔗 Constraint-based URL building
- ⚡ Efficient typed array data transfer
- 🛡️ Robust error handling

## Dataset Information

All examples use the same test dataset:
- **URL**: `https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap`
- **Type**: GFS (Global Forecast System) meteorological data
- **Variables**: Temperature (t2m), Total Cloud Cover (tcc), Wind Gust (gust), and more
- **Coordinates**: time, latitude, longitude

## Troubleshooting

### WASM Loading Issues
- Ensure you're running from a web server, not opening files directly
- Check browser console for specific error messages
- Try the development server: `bun run serve.js`

### Network Issues
- The dataset URL must be accessible from your location
- Some corporate firewalls may block the requests
- Try running the Bun example first to test connectivity

### Browser Compatibility
- Modern browsers with WebAssembly support required
- Chrome 57+, Firefox 52+, Safari 11+, Edge 16+
- Enable JavaScript if disabled

### Performance
- Large data selections may take time to load
- Start with small ranges (lat/lon 0-10) for testing
- The development server includes CORS and caching headers for optimal performance

## Development

To modify or extend these examples:

1. Build the WASM package first:
   ```bash
   cd readap-wasm
   wasm-pack build
   ```

2. The built package will be in `pkg/` directory

3. Examples import from `../pkg/readap_wasm.js`

4. Test your changes with the development server

## Support

If you encounter issues:
1. Check the browser console for error messages
2. Ensure the WASM package is built (`wasm-pack build`)
3. Try the development server for web-based examples
4. Verify network connectivity with the Bun example
