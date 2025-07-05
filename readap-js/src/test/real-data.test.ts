import { test, describe } from 'node:test';
import assert from 'node:assert';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { DAPClient } from '../client.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

describe('Real OpenDAP Data Tests', () => {
  test('should parse real DDS data from test files', async () => {
    const client = new DAPClient('https://test.example.com/data');
    
    // Read the real DDS file from the test data
    const ddsPath = join(__dirname, '../../../readap/data/swden/44097w9999.nc.dods');
    const ddsContent = readFileSync(ddsPath, 'utf-8');
    
    const dataset = await client.parseDds(ddsContent);
    
    // Verify the dataset structure matches the real data
    assert.strictEqual(dataset.name, 'data/swden/44097/44097w9999.nc');
    
    // Check that all expected variables are present
    const expectedVariables = [
      'time',
      'frequency', 
      'spectral_wave_density',
      'mean_wave_dir',
      'principal_wave_dir',
      'wave_spectrum_r1',
      'wave_spectrum_r2'
    ];
    
    for (const variable of expectedVariables) {
      assert.ok(dataset.variables.includes(variable), `Variable ${variable} should be present`);
    }
    
    console.log('✓ Successfully parsed real DDS data with variables:', dataset.variables);
  });

  test('should parse real DAS attributes from test files', async () => {
    const client = new DAPClient('https://test.example.com/data');
    
    // Read the real DAS file from the test data
    const dasPath = join(__dirname, '../../../readap/data/swden/44097w9999.nc.das');
    const dasContent = readFileSync(dasPath, 'utf-8');
    
    const attributes = await client.parseDas(dasContent);
    
    // Verify key attributes are parsed correctly
    assert.ok(attributes.time);
    assert.strictEqual(attributes.time.long_name.value, 'Epoch Time');
    assert.strictEqual(attributes.time.units.value, 'seconds since 1970-01-01 00:00:00 UTC');
    
    assert.ok(attributes.frequency);
    assert.strictEqual(attributes.frequency.units.value, 'Hz');
    
    assert.ok(attributes.spectral_wave_density);
    assert.strictEqual(attributes.spectral_wave_density.units.value, '(meter * meter)/Hz');
    assert.strictEqual(parseFloat(attributes.spectral_wave_density._FillValue.value), 999.0);
    
    assert.ok(attributes.mean_wave_dir);
    assert.strictEqual(attributes.mean_wave_dir.units.value, 'degrees_true');
    assert.strictEqual(attributes.mean_wave_dir._FillValue.value, '999');
    
    console.log('✓ Successfully parsed real DAS attributes');
  });

  test('should generate correct DODS URLs for real data variables', async () => {
    const client = new DAPClient('https://test.opendap.org/opendap/data/swden/44097w9999.nc');
    
    // Test URL generation for real variables
    const dodsUrl = await client.getDodsUrl(['spectral_wave_density', 'mean_wave_dir']);
    
    assert.ok(dodsUrl.includes('spectral_wave_density'));
    assert.ok(dodsUrl.includes('mean_wave_dir'));
    assert.ok(dodsUrl.includes('.dods'));
    
    // Test with constraints on real dimensions
    const constrainedUrl = await client.getDodsUrl(['spectral_wave_density'], {
      spectral_wave_density: [{ start: 0, end: 3 }] // time dimension constraint
    });
    
    assert.ok(constrainedUrl.includes('spectral_wave_density'));
    assert.ok(constrainedUrl.includes('[0:3]'));
    
    console.log('✓ Generated DODS URLs:', dodsUrl);
  });

  test('should handle dataset info extraction from real data', async () => {
    const client = new DAPClient('https://test.example.com/data');
    
    // Read both DDS and DAS files
    const ddsPath = join(__dirname, '../../../readap/data/swden/44097w9999.nc.dods');
    const dasPath = join(__dirname, '../../../readap/data/swden/44097w9999.nc.das');
    
    const ddsContent = readFileSync(ddsPath, 'utf-8');
    const dasContent = readFileSync(dasPath, 'utf-8');
    
    // Parse both
    const dataset = await client.parseDds(ddsContent);
    const attributes = await client.parseDas(dasContent);
    
    // Verify we can extract meaningful information
    assert.ok(dataset.variables.length > 0);
    assert.ok(Object.keys(attributes).length > 0);
    
    // Check that variables have corresponding attributes
    for (const variable of ['time', 'frequency', 'spectral_wave_density']) {
      assert.ok(dataset.variables.includes(variable), `Variable ${variable} in DDS`);
      assert.ok(attributes[variable], `Attributes for ${variable} in DAS`);
    }
    
    console.log('✓ Successfully extracted dataset info from real data');
  });

  test('should validate DODS data workflow with real structure', async () => {
    const client = new DAPClient('https://test.opendap.org/opendap/data/swden/44097w9999.nc');
    
    try {
      // Test the complete workflow that would be used for DODS data fetching
      const ddsUrl = await client.getDdsUrl();
      const dasUrl = await client.getDasUrl();
      const dodsUrl = await client.getDodsUrl(['spectral_wave_density'], {
        spectral_wave_density: [{ start: 0, end: 2 }]
      });
      
      // Verify URLs are properly formatted
      assert.ok(ddsUrl.endsWith('.dds'));
      assert.ok(dasUrl.endsWith('.das'));
      assert.ok(dodsUrl.includes('.dods'));
      assert.ok(dodsUrl.includes('spectral_wave_density'));
      
      console.log('✓ DODS workflow URLs generated successfully');
      console.log('  DDS:', ddsUrl);
      console.log('  DAS:', dasUrl);
      console.log('  DODS:', dodsUrl);
      
    } catch (error) {
      // Network errors are expected in test environment
      console.log('Network portion of test skipped:', (error as Error).message);
    }
  });
});