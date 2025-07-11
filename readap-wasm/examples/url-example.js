// Example usage of the readap-wasm bindings

import init, { UrlBuilder, IndexRange, IndexRangeType } from './pkg/readap_wasm.js';

async function main() {
    // Initialize the WASM module
    await init();

    // Basic URL construction
    const builder = new UrlBuilder("https://example.com/data/dataset");
    console.log("DAS URL:", builder.dasUrl());
    console.log("DDS URL:", builder.ddsUrl());
    console.log("DODS URL:", builder.dodsUrl());

    // Variable selection
    const builder2 = new UrlBuilder("https://example.com/data/ocean")
        .addVariable("temperature")
        .addVariable("salinity");
    console.log("Variables URL:", builder2.dodsUrl());

    // Single index constraint
    const builder3 = new UrlBuilder("https://example.com/data/ocean")
        .addVariable("temperature")
        .addSingleIndex("temperature", 5);
    console.log("Single index URL:", builder3.dodsUrl());

    // Range constraint
    const builder4 = new UrlBuilder("https://example.com/data/ocean")
        .addVariable("temperature")
        .addRange("temperature", 0, 10, null);
    console.log("Range URL:", builder4.dodsUrl());

    // Range with stride
    const builder5 = new UrlBuilder("https://example.com/data/ocean")
        .addVariable("temperature")
        .addRange("temperature", 0, 20, 2);
    console.log("Range with stride URL:", builder5.dodsUrl());

    // Multi-dimensional constraint using IndexRange objects
    const indices = [
        IndexRange.fromRange(0, 10, null),      // time dimension
        new IndexRange(5),                      // depth dimension (single value)
        IndexRange.fromRange(20, 50, 2),        // latitude dimension with stride
        IndexRange.fromRange(-180, 180, null)   // longitude dimension
    ];
    
    const builder6 = new UrlBuilder("https://example.com/data/ocean")
        .addVariable("temperature")
        .addIndexConstraint("temperature", indices);
    console.log("Multi-dimensional URL:", builder6.dodsUrl());

    // Multiple variables with different constraints
    const builder7 = new UrlBuilder("https://example.com/data/ocean")
        .addVariable("temperature")
        .addVariable("salinity")
        .addRange("temperature", 0, 10, null)
        .addSingleIndex("salinity", 3);
    console.log("Multiple variables with constraints:", builder7.dodsUrl());

    // Clean up (free WASM memory)
    indices.forEach(idx => idx.free());
    builder.free();
    builder2.free();
    builder3.free();
    builder4.free();
    builder5.free();
    builder6.free();
    builder7.free();
}

main().catch(console.error);
