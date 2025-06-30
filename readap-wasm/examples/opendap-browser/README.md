# OpenDAP Dataset Browser

A React web application that demonstrates how to use the `@mattnucc/readap` package to browse and visualize OpenDAP datasets in the browser.

## Features

- 🌐 **Browse OpenDAP Datasets**: Load datasets from any OpenDAP server
- 📊 **Variable Exploration**: View all variables with their types, dimensions, and attributes
- 🎯 **Index Selection**: Select specific indices for multidimensional data
- 📈 **Data Visualization**: View fetched data with statistics and simple charts
- 📱 **Responsive Design**: Works on desktop and mobile devices

## Getting Started

### Prerequisites

- Node.js 16+ 
- npm or yarn

### Installation

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

The application will be available at `http://localhost:5173`

### Building for Production

```bash
# Build for production
npm run build

# Preview production build
npm run preview
```

## Usage

1. **Enter Dataset URL**: Input an OpenDAP dataset URL in the text field
2. **Load Dataset**: Click "Load Dataset" to fetch metadata
3. **Select Variable**: Choose a variable from the list to explore
4. **Set Indices**: For multidimensional variables, select specific indices for each dimension
5. **Fetch Data**: Click "Fetch Data" to retrieve and visualize the selected data slice

### Example Datasets

The app includes two example datasets:

- **NOAA Chlorophyll-a**: Small dataset good for testing
- **Earthmover GFS Solar**: Large meteorological dataset

## How It Works

The application uses the `@mattnucc/readap` WebAssembly package to:

1. Initialize the WASM module in the browser
2. Use `ImmutableDataset.fromURL()` to load OpenDAP metadata
3. Use `SimpleConstraintBuilder` to create index-based selections
4. Fetch data using the dataset's `getVariable()` method with constraints

## Components

- **DatasetBrowser**: Main component managing state and WASM initialization
- **VariableSelector**: UI for choosing variables and setting dimension indices  
- **DataVisualization**: Display fetched data with statistics and simple visualizations

## Package Usage Example

```javascript
import init, { ImmutableDataset, SimpleConstraintBuilder } from '@mattnucc/readap'

// Initialize WASM
await init()

// Load dataset
const dataset = await ImmutableDataset.fromURL('https://example.com/data.nc')

// Build constraints
const builder = new SimpleConstraintBuilder()
  .add_single('time', 0)
  .add_single('latitude', 100)
  .add_single('longitude', 200)

// Fetch data
const data = await dataset.getVariable('temperature', builder.build())
console.log(data.data) // Float64Array with values
```

## Dependencies

- **React 18**: UI framework
- **Vite**: Build tool and dev server
- **@mattnucc/readap**: WebAssembly OpenDAP client

## Browser Compatibility

The application requires a modern browser with:
- WebAssembly support
- ES2018+ features
- Fetch API

Supported browsers:
- Chrome 69+
- Firefox 78+
- Safari 14+
- Edge 79+

## License

This example is part of the readap-wasm project and follows the same license terms.
