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

      // Calculate variable sizes and identify coordinate variables (skip coordinate data fetching for now)
      for (const [varName, varInfo] of Object.entries(variablesInfo)) {
        // Calculate total size
        const totalSize = varInfo.dimensions 
          ? varInfo.dimensions.reduce((acc, dim) => acc * dim.size, 1)
          : 1
        enhancedMetadata.variableSizes[varName] = totalSize

        // Mark coordinate variables but don't fetch data yet
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

      // Try to fetch coordinate samples in the background (non-blocking)
      setTimeout(async () => {
        for (const [varName, coordInfo] of Object.entries(enhancedMetadata.coordinates)) {
          if (coordInfo.values.length === 0 && coordInfo.size > 0) {
            try {
              console.log(`Background fetching coordinate sample for ${varName}...`)
              const coordData = await Promise.race([
                ds.getVariable(varName, `${varName}[0:1:${Math.min(2, coordInfo.size - 1)}]`),
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
      }, 100) // Start background fetching after a small delay

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

      // Fetch the data - handle both with and without constraints, with timeout
      let varData
      const fetchPromise = constraint && constraint.trim() !== '' 
        ? dataset.getVariable(selectedVariable, constraint)
        : dataset.getVariable(selectedVariable)
      
      const timeoutPromise = new Promise((_, reject) => 
        setTimeout(() => reject(new Error('Data fetch timeout after 15 seconds')), 15000)
      )

      console.log(`Fetching ${selectedVariable}${constraint ? ` with constraint: ${constraint}` : ' without constraints'}`)
      varData = await Promise.race([fetchPromise, timeoutPromise])
      
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