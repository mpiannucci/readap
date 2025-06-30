// Example demonstrating the fetch-enabled OpenDAP WASM library
import init, { OpenDAPDataset } from '../pkg/readap_wasm.js';

async function main() {
    // Initialize the WASM module
    await init();
    
    try {
        // Create a dataset with automatic metadata fetching
        console.log('Loading dataset metadata...');
        const dataset = await OpenDAPDataset.fromURL('https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap');
        
        console.log('Available variables:', dataset.getVariableNames());
        
        // Get variable information
        const tempInfo = JSON.parse(dataset.getVariableInfo('t2m'));
        console.log('Temperature variable:', tempInfo);
        
        // Simple data access - no constraints
        console.log('Fetching temperature data...');
        const tempData = await dataset.getVariable('t2m');
        console.log('Temperature data type:', tempData.type);
        console.log('Temperature data length:', tempData.length);
        console.log('First 10 values:', tempData.data.slice(0, 10));
        
        // Index-based selection (isel)
        console.log('Fetching with index-based selection...');
        const indexSelection = dataset.isel({
            time: { type: "single", value: 0 },
            latitude: { type: "range", start: 10, end: 20 }
        });
        const tempSlice = await dataset.getVariable('t2m', indexSelection);
        console.log('Temperature slice shape:', tempSlice.length);
        
        // Value-based selection (sel) - requires coordinate loading
        console.log('Loading coordinates for value-based selection...');
        await dataset.loadCoordinates('time');
        await dataset.loadCoordinates('latitude');
        await dataset.loadCoordinates('longitude');
        
        const valueSelection = dataset.sel({
            time: "2023-01-15",  // nearest neighbor
            latitude: [40.0, 50.0],   // range selection
            longitude: -74.0           // single value nearest neighbor
        });
        
        console.log('Fetching with value-based selection...');
        const tempSelected = await dataset.getVariable('t2m', valueSelection);
        console.log('Selected temperature data:', tempSelected);
        
        // Multiple variables at once
        console.log('Fetching multiple variables...');
        const varNames = ['t2m', 'tcc', 'gust'];
        const multiData = await dataset.getVariables(varNames, indexSelection);
        
        Object.keys(multiData).forEach(varName => {
            console.log(`${varName}: ${multiData[varName].type} with ${multiData[varName].length} elements`);
        });
        
        // Chained selections
        const chainedSelection = dataset
            .isel({ time: { type: "single", value: 0 } })
            .sel({ latitude: [40.0, 50.0] });
        
        const chainedData = await dataset.getVariable('t2m', chainedSelection);
        console.log('Chained selection result:', chainedData);
        
    } catch (error) {
        console.error('Error:', error);
    }
}

// Alternative lazy loading approach
async function lazyExample() {
    await init();
    
    try {
        // Create dataset without automatic metadata loading
        const dataset = OpenDAPDataset.fromURLLazy('https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap');
        
        // Manually load metadata when needed
        await dataset.parseDAS(await fetch(dataset.dasUrl()).then(r => r.text()));
        await dataset.parseDDS(await fetch(dataset.ddsUrl()).then(r => r.text()));
        
        console.log('Variables:', dataset.getVariableNames());
        
        // Manual DODS parsing
        const dodsUrl = dataset.dodsUrl('t2m[0:10]');
        const dodsResponse = await fetch(dodsUrl);
        const dodsData = new Uint8Array(await dodsResponse.arrayBuffer());
        const parsedData = dataset.parseDODS(dodsData);
        
        console.log('Manually parsed data:', parsedData);
        
    } catch (error) {
        console.error('Error in lazy example:', error);
    }
}

// Run the examples
main().then(() => {
    console.log('Main example completed');
    return lazyExample();
}).then(() => {
    console.log('Lazy example completed');
}).catch(console.error);
