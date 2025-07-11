// Example usage of the DAS parser WASM bindings
// This demonstrates how to use parseDasAttributes with JavaScript

// Assuming the WASM module is loaded as `readap`

const dasString = `Attributes {
    time {
        String long_name "Epoch Time";
        String short_name "time";
        String standard_name "time";
        String units "seconds since 1970-01-01 00:00:00 UTC";
    }
    frequency {
        String long_name "Frequency";
        String short_name "frequency";
        String standard_name "frequency";
        String units "Hz";
    }
    spectral_wave_density {
        String long_name "Spectral Wave Density";
        String short_name "swden";
        String standard_name "spectral_wave_density";
        String units "(meter * meter)/Hz";
        Float32 _FillValue 999.0;
        Int32 valid_min 0;
        Int32 valid_max 1000;
    }
}`;

// Parse the DAS attributes
try {
    const dasAttributes = readap.parseDasAttributes(dasString);
    
    // Access nested structure naturally
    console.log("Time variable attributes:");
    console.log(`  long_name: ${dasAttributes.time.long_name.value}`);
    console.log(`  units: ${dasAttributes.time.units.value}`);
    
    // Iterate over variables
    console.log("\nAll variables and their attributes:");
    for (const [varName, variable] of Object.entries(dasAttributes)) {
        console.log(`\n${varName}:`);
        for (const [attrName, attribute] of Object.entries(variable)) {
            console.log(`  ${attrName} (${attribute.dataType}): ${attribute.value}`);
        }
    }
    
    // Type-safe access to numeric values
    const fillValue = dasAttributes.spectral_wave_density._FillValue.value;
    console.log(`\nFill value for spectral_wave_density: ${fillValue}`);
    console.log(`Type of fill value: ${typeof fillValue}`); // "number"
    
    // Check data types
    if (dasAttributes.spectral_wave_density._FillValue.dataType === "Float32") {
        console.log("Fill value is a Float32");
    }
    
} catch (error) {
    console.error("Failed to parse DAS attributes:", error);
}