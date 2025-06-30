import { useState, useEffect, useCallback } from 'react'
import init, { ImmutableDataset, SimpleConstraintBuilder } from '@mattnucc/readap'
import VariableSelector from './VariableSelector'
import DataVisualization from './DataVisualization'

const DEFAULT_URL = 'https://coastwatch.pfeg.noaa.gov/erddap/griddap/erdMBchla1day'

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

      // Parse metadata
      const variablesInfo = JSON.parse(ds.getVariablesInfo())
      const varNames = ds.getVariableNames()
      
      // Enhanced metadata with coordinate information
      const enhancedMetadata = {
        url,
        variables: variablesInfo,
        variableNames: varNames,
        coordinates: {},
        variableSizes: {}
      }

      // Calculate variable sizes and identify coordinate variables
      for (const [varName, varInfo] of Object.entries(variablesInfo)) {
        // Calculate total size
        const totalSize = varInfo.dimensions 
          ? varInfo.dimensions.reduce((acc, dim) => acc * dim.size, 1)
          : 1
        enhancedMetadata.variableSizes[varName] = totalSize

        // If this looks like a coordinate variable (1D, name matches dimension)
        if (varInfo.dimensions?.length === 1 && 
            varInfo.dimensions[0].name === varName) {
          try {
            // Try to fetch a small sample of coordinate values for display
            console.log(`Fetching coordinate sample for ${varName}...`)
            const coordData = await ds.getVariable(varName, `${varName}[0:1:${Math.min(4, varInfo.dimensions[0].size - 1)}]`)
            enhancedMetadata.coordinates[varName] = {
              values: Array.from(coordData.data),
              size: varInfo.dimensions[0].size,
              attributes: varInfo.attributes
            }
          } catch (err) {
            console.warn(`Could not fetch coordinate data for ${varName}:`, err.message)
            enhancedMetadata.coordinates[varName] = {
              values: [],
              size: varInfo.dimensions[0].size,
              attributes: varInfo.attributes
            }
          }
        }
      }

      setMetadata(enhancedMetadata)

      console.log('Dataset loaded successfully:', varNames)
    } catch (err) {
      console.error('Failed to load dataset:', err)
      onError(`Failed to load dataset: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }, [url, wasmInitialized, onError])

  const fetchData = useCallback(async () => {
    if (!dataset || !selectedVariable || !selectedIndices) return

    setLoading(true)
    onError(null)

    try {
      // Build constraint using SimpleConstraintBuilder
      let builder = new SimpleConstraintBuilder()
      
      // Add constraints for each dimension
      Object.entries(selectedIndices).forEach(([dim, value]) => {
        if (value !== undefined && value !== null) {
          console.log(`Adding constraint: ${dim}[${value}]`)
          builder = builder.addSingle(dim, value)
        }
      })

      const constraint = builder.build()
      console.log('Built constraint string:', constraint)
      console.log('Selected indices:', selectedIndices)

      // Fetch the data - handle both with and without constraints
      let varData
      if (constraint && constraint.trim() !== '') {
        console.log(`Fetching ${selectedVariable} with constraint: ${constraint}`)
        varData = await dataset.getVariable(selectedVariable, constraint)
      } else {
        console.log(`Fetching ${selectedVariable} without constraints (scalar or no selection)`)
        varData = await dataset.getVariable(selectedVariable)
      }
      
      setData({
        variable: selectedVariable,
        data: Array.from(varData.data),
        dimensions: varData.dimensions,
        attributes: varData.attributes
      })

      console.log('Data fetched successfully:', varData)
    } catch (err) {
      console.error('Failed to fetch data:', err)
      onError(`Failed to fetch data: ${err.message}`)
    } finally {
      setLoading(false)
    }
  }, [dataset, selectedVariable, selectedIndices, onError])

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