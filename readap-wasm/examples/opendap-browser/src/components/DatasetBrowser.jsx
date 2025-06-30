import { useState, useEffect, useCallback } from 'react'
import init, { ImmutableDataset, SimpleConstraintBuilder, universalFetchText } from '@mattnucc/readap'
import VariableSelector from './VariableSelector'
import DataVisualization from './DataVisualization'

const DEFAULT_URL = 'https://coastwatch.pfeg.noaa.gov/erddap/griddap/erdMBchla1day'

/**
 * Parse DDS content to extract dimensions and variable information
 * This is a workaround for the library's broken dimension parsing
 */
function parseDDS(ddsContent) {
  const dimensions = new Map()
  const variables = {}
  
  const lines = ddsContent.split('\n')
  
  for (const line of lines) {
    const trimmed = line.trim()
    
    // Parse coordinate variables: Float64 longitude[longitude = 1440];
    const coordMatch = trimmed.match(/^(Float\d+|Int\d+)\s+(\w+)\[(\w+)\s*=\s*(\d+)\];$/)
    if (coordMatch) {
      const [, dataType, varName, dimName, size] = coordMatch
      const dimSize = parseInt(size)
      
      dimensions.set(dimName, dimSize)
      variables[varName] = {
        isCoordinate: true,
        dimensions: [{ name: dimName, size: dimSize }],
        shape: [dimSize]
      }
      continue
    }
    
    // Parse grid array declarations: Float32 gust[longitude = 1440][latitude = 721][time = 1138][step = 209];
    const gridMatch = trimmed.match(/^(Float\d+|Int\d+)\s+(\w+)(\[.+\]);$/)
    if (gridMatch) {
      const [, dataType, varName, dimString] = gridMatch
      
      // Extract all dimensions from [longitude = 1440][latitude = 721]...
      const dimMatches = [...dimString.matchAll(/\[(\w+)\s*=\s*(\d+)\]/g)]
      const varDimensions = []
      const shape = []
      
      for (const [, dimName, size] of dimMatches) {
        const dimSize = parseInt(size)
        dimensions.set(dimName, dimSize)
        varDimensions.push({ name: dimName, size: dimSize })
        shape.push(dimSize)
      }
      
      variables[varName] = {
        isCoordinate: false,
        dimensions: varDimensions,
        shape: shape
      }
    }
  }
  
  return { dimensions, variables }
}

function DatasetBrowser({ onError }) {
  const [wasmInitialized, setWasmInitialized] = useState(false)
  const [url, setUrl] = useState(DEFAULT_URL)
  const [loading, setLoading] = useState(false)
  const [dataset, setDataset] = useState(null)
  const [metadata, setMetadata] = useState(null)
  const [selectedVariable, setSelectedVariable] = useState(null)
  const [selectedIndices, setSelectedIndices] = useState({})
  const [data, setData] = useState(null)

  // Initialize WASM on component mount
  useEffect(() => {
    init().then(() => {
      setWasmInitialized(true)
      console.log('WASM initialized successfully')
    }).catch(err => {
      onError(`Failed to initialize WASM: ${err.message}`)
    })
  }, [onError])

  const loadDataset = useCallback(async () => {
    if (!wasmInitialized) return

    setLoading(true)
    setDataset(null)
    setMetadata(null)
    setSelectedVariable(null)
    setData(null)
    onError(null)

    try {
      console.log('Loading dataset:', url)
      const ds = await ImmutableDataset.fromURL(url)
      setDataset(ds)

      // Get DDS content and parse it directly (the library's getVariablesInfo has broken dimension parsing)
      console.log('Fetching DDS to extract dimension information...')
      const ddsContent = await universalFetchText(ds.ddsUrl())
      const { dimensions, variables } = parseDDS(ddsContent)
      
      // Get basic variable info from the library (for data types, etc.)
      const variablesInfoJson = ds.getVariablesInfo()
      const basicVarInfo = JSON.parse(variablesInfoJson)
      
      // Merge parsed DDS info with basic variable info
      for (const [varName, varData] of Object.entries(variables)) {
        if (basicVarInfo[varName]) {
          basicVarInfo[varName].dimensions = varData.dimensions
          basicVarInfo[varName].shape = varData.shape
        }
      }

      // Enhanced metadata with coordinate information
      const enhancedMetadata = {
        url,
        variables: basicVarInfo,
        variableNames: ds.getVariableNames(),
        coordinates: {},
        variableSizes: {},
        dimensions: dimensions
      }

      // Calculate variable sizes and identify coordinate variables
      for (const [varName, varInfo] of Object.entries(basicVarInfo)) {
        // Calculate total size
        const totalSize = varInfo.dimensions 
          ? varInfo.dimensions.reduce((acc, dim) => acc * dim.size, 1)
          : 1
        enhancedMetadata.variableSizes[varName] = totalSize

        // Mark coordinate variables
        if (varInfo.dimensions?.length === 1 && 
            varInfo.dimensions[0].name === varName) {
          enhancedMetadata.coordinates[varName] = {
            values: [], // We'll fetch these on demand
            size: varInfo.dimensions[0].size,
            attributes: varInfo.attributes,
            isCoordinate: true
          }
        }
      }

      setMetadata(enhancedMetadata)

      // Try to fetch coordinate samples in the background (non-blocking)
      setTimeout(async () => {
        for (const [varName, coordInfo] of Object.entries(enhancedMetadata.coordinates)) {
          if (coordInfo.values.length === 0 && coordInfo.size > 0) {
            try {
              console.log(`Background fetching coordinate sample for ${varName}...`)
              
              // Fetch just a few coordinate values for preview
              const maxSample = Math.min(4, coordInfo.size - 1)
              const constraint = maxSample > 0 ? `${varName}[0:1:${maxSample}]` : `${varName}[0]`
              
              const coordData = await Promise.race([
                ds.getVariable(varName, constraint),
                new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 3000))
              ])
              
              // Update metadata with fetched values
              setMetadata(prev => ({
                ...prev,
                coordinates: {
                  ...prev.coordinates,
                  [varName]: {
                    ...prev.coordinates[varName],
                    values: Array.from(coordData.data)
                  }
                }
              }))
              console.log(`✓ Background fetched ${coordData.data.length} coordinate values for ${varName}`)
            } catch (err) {
              console.warn(`Background coordinate fetch failed for ${varName}:`, err.message)
            }
          }
        }
      }, 1000) // Start background fetching after a delay

      console.log('Dataset loaded successfully:', varNames)
    } catch (err) {
      console.error('Failed to load dataset:', err)
      onError(`Failed to load dataset: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }, [url, wasmInitialized, onError])

  const fetchData = useCallback(async () => {
    if (!dataset || !selectedVariable || !metadata) return

    setLoading(true)
    onError(null)

    try {
      const varInfo = metadata.variables[selectedVariable]
      if (!varInfo) {
        throw new Error(`Variable ${selectedVariable} not found`)
      }

      let constraint = ''
      
      // Always build a constraint for safety
      if (varInfo.dimensions && varInfo.dimensions.length > 0) {
        // Build constraint using SimpleConstraintBuilder
        let builder = new SimpleConstraintBuilder()
        
        // Add constraints for each dimension
        varInfo.dimensions.forEach(dim => {
          const selectedIndex = selectedIndices[dim.name]
          if (selectedIndex !== undefined && selectedIndex !== null) {
            console.log(`Adding constraint: ${dim.name}[${selectedIndex}]`)
            builder = builder.addSingle(dim.name, selectedIndex)
          } else {
            // Default to first index if none selected
            console.log(`Adding default constraint: ${dim.name}[0]`)
            builder = builder.addSingle(dim.name, 0)
          }
        })

        constraint = builder.build()
      }

      console.log('Built constraint string:', constraint)
      console.log('Selected indices:', selectedIndices)
      console.log('Variable dimensions:', varInfo.dimensions)

      // Always use constraints for safety - never fetch without them for large variables
      const estimatedSize = varInfo.dimensions 
        ? varInfo.dimensions.reduce((acc, dim) => acc * dim.size, 1)
        : 1

      let varData
      if (constraint && constraint.trim() !== '') {
        console.log(`Fetching ${selectedVariable} with constraint: ${constraint}`)
        varData = await Promise.race([
          dataset.getVariable(selectedVariable, constraint),
          new Promise((_, reject) => 
            setTimeout(() => reject(new Error('Data fetch timeout after 10 seconds')), 10000)
          )
        ])
      } else if (estimatedSize <= 1000) {
        // Only fetch without constraints for very small variables
        console.log(`Fetching small variable ${selectedVariable} without constraints (${estimatedSize} elements)`)
        varData = await Promise.race([
          dataset.getVariable(selectedVariable),
          new Promise((_, reject) => 
            setTimeout(() => reject(new Error('Data fetch timeout after 10 seconds')), 10000)
          )
        ])
      } else {
        throw new Error(`Variable too large (${estimatedSize} elements) - constraints required`)
      }
      
      setData({
        variable: selectedVariable,
        data: Array.from(varData.data),
        dimensions: varData.dimensions,
        attributes: varData.attributes,
        constraint: constraint
      })

      console.log('Data fetched successfully:', {
        variable: selectedVariable,
        dataLength: varData.data.length,
        constraint: constraint
      })
    } catch (err) {
      console.error('Failed to fetch data:', err)
      onError(`Failed to fetch data: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }, [dataset, selectedVariable, selectedIndices, metadata, onError])

  return (
    <div className="dataset-browser">
      <section className="url-input">
        <h2>Dataset URL</h2>
        <div className="input-group">
          <input
            type="text"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="Enter OpenDAP dataset URL"
            disabled={!wasmInitialized || loading}
          />
          <button 
            onClick={loadDataset} 
            disabled={!wasmInitialized || loading || !url}
          >
            {loading ? 'Loading...' : 'Load Dataset'}
          </button>
        </div>
        <div className="example-urls">
          <p>Example URLs:</p>
          <ul>
            <li>
              <a href="#" onClick={() => setUrl('https://coastwatch.pfeg.noaa.gov/erddap/griddap/erdMBchla1day')}>
                NOAA Chlorophyll-a (small dataset)
              </a>
            </li>
            <li>
              <a href="#" onClick={() => setUrl('https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap')}>
                Earthmover GFS Solar (large dataset)
              </a>
            </li>
          </ul>
        </div>
      </section>

      {metadata && (
        <>
          <VariableSelector
            metadata={metadata}
            selectedVariable={selectedVariable}
            selectedIndices={selectedIndices}
            onVariableSelect={setSelectedVariable}
            onIndicesChange={setSelectedIndices}
            onFetchData={fetchData}
            loading={loading}
          />

          {data && (
            <DataVisualization
              data={data}
              metadata={metadata}
            />
          )}
        </>
      )}

      {!wasmInitialized && (
        <div className="loading">Initializing WebAssembly...</div>
      )}
    </div>
  )
}

export default DatasetBrowser