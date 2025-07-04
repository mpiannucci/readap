# readap-wasm API Updates

This document summarizes the updates made to the WASM bindings to align with the actual readap crate API.

## Key Issues Fixed

### 1. Incorrect Type Names
- **Before**: Used non-existent types like `Dataset`, `Attribute`, `Variable`, `CoordinateVariable`, `Value`, `VariableType`
- **After**: Uses actual types from the readap crate:
  - `DdsDataset` - The main dataset structure from DDS parsing
  - `DasAttribute` - Individual attributes from DAS parsing
  - `DdsValue` - Union type for Array, Grid, Structure, Sequence values
  - `DataType`, `DataValue` - From the `data` module
  - `VariableInfo`, `CoordinateInfo` - From the `query` module

### 2. Incorrect Function Signatures
- **Before**: Called non-existent parsing functions like `readap::parse_dds()`
- **After**: Uses correct parsing methods:
  - `DdsDataset::from_bytes()` for DDS parsing
  - `parse_das_attributes()` for DAS parsing
  - `DodsDataset::from_bytes()` for DODS parsing

### 3. UrlBuilder API Corrections
- **Before**: Assumed mutable methods and incorrect function signatures
- **After**: Uses correct builder pattern where methods consume `self` and return new instances:
  - `UrlBuilder::new()` creates a new builder (no Result return)
  - Methods like `add_variable()`, `add_range()` consume self and return new builder
  - `dods_url()` returns `Result<String, Error>`, not `String`

### 4. Improved Data Structure Conversion
- **Before**: Tried to convert non-existent array structures
- **After**: Properly handles the actual DDS structure:
  - Converts `DdsValue` enum variants (Array, Grid, Structure, Sequence)
  - Handles coordinate information from grids and arrays
  - Properly extracts metadata like variable lists and coordinate lists

## New API Features

### Enhanced Parsing Functions
```javascript
// Parse DDS and get structured information
const dataset = parse_dds(ddsContent);
console.log(dataset.variables);  // List of variable names
console.log(dataset.coordinates); // List of coordinate names
console.log(dataset.values);     // Detailed DDS values

// Parse DAS with proper structure
const attributes = parse_das(dasContent);
// Returns: { [variableName]: { [attributeName]: DasAttribute } }
```

### Metadata Query Functions
```javascript
// Get detailed variable information
const varInfo = get_variable_info(ddsContent, 'temperature');
// Returns: { name, dataType, variableType, coordinates, dimensions }

// Get coordinate information
const coordInfo = get_coordinate_info(ddsContent, 'time');
// Returns: { name, dataType, size, variablesUsing }
```

### Improved URL Builder
```javascript
// Correct builder pattern usage
const url = new JsUrlBuilder(baseUrl)
  .addVariable('temperature')
  .addRange('temperature', 0, 10)
  .addSingleIndex('temperature', 5)
  .dodsUrl();

// Multidimensional constraints
const url2 = new JsUrlBuilder(baseUrl)
  .addVariable('temperature')
  .addMultidimensionalConstraint('temperature', [
    { start: 0, end: 10 },           // time
    5,                               // latitude (single index)
    { start: 0, end: 8, stride: 2 }  // longitude with stride
  ])
  .dodsUrl();
```

## Data Type Mapping

### DataType Enum
- `Byte`, `Int16`, `UInt16`, `Int32`, `UInt32`
- `Float32`, `Float64`
- `String`, `URL`

### DdsValue Types
- **Array**: Simple n-dimensional arrays with coordinates
- **Grid**: Arrays with associated coordinate arrays
- **Structure**: Nested data structures
- **Sequence**: Variable-length sequences

### Variable Types (from query module)
- `Array`, `Grid`, `Structure`, `Sequence`

## Breaking Changes

1. **Function Names**: All parsing functions now match the actual API
2. **Return Types**: Functions return structured objects matching the Rust types
3. **Builder Pattern**: UrlBuilder methods now consume self and return new instances
4. **Error Handling**: Proper error handling for functions that can fail
5. **Type Structure**: DAS attributes are now properly nested by variable name

## TypeScript Support

Added `types.d.ts` with comprehensive type definitions for:
- All exported functions
- Data structures returned by parsing functions
- UrlBuilder class with correct method signatures
- Interfaces for variable and coordinate information

## Example Usage

See `example.js` for comprehensive usage examples showing:
- Basic DDS/DAS parsing
- Variable and coordinate information queries
- URL building with various constraint types
- Error handling patterns

This update makes the WASM bindings actually functional and aligned with the real readap crate API.