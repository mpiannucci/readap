import { useEffect, useState } from 'react'

function VariableSelector({ 
  metadata, 
  selectedVariable, 
  selectedIndices, 
  onVariableSelect, 
  onIndicesChange, 
  onFetchData,
  loading 
}) {
  const variables = metadata.variables
  const variableNames = metadata.variableNames
  const coordinates = metadata.coordinates || {}
  const variableSizes = metadata.variableSizes || {}
  const [showCoordinates, setShowCoordinates] = useState({})

  // Reset ranges when variable changes
  useEffect(() => {
    if (selectedVariable && variables[selectedVariable]) {
      const varInfo = variables[selectedVariable]
      const defaultRanges = {}
      
      if (varInfo.dimensions) {
        varInfo.dimensions.forEach(dim => {
          // Default to first few elements for safety
          const maxSample = Math.min(10, dim.size - 1)
          defaultRanges[dim.name] = {
            min: 0,
            max: maxSample
          }
        })
      }
      
      onIndicesChange(defaultRanges)
    }
  }, [selectedVariable, variables, onIndicesChange])

  const handleRangeChange = (dimName, type, value) => {
    const numValue = parseInt(value, 10)
    if (!isNaN(numValue)) {
      onIndicesChange(prev => ({
        ...prev,
        [dimName]: {
          ...prev[dimName],
          [type]: numValue
        }
      }))
    }
  }

  const formatBytes = (size) => {
    if (size === 0) return '0 B'
    if (size === 1) return '1 value'
    if (size < 1024) return `${size} values`
    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)}K values`
    if (size < 1024 * 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)}M values`
    return `${(size / (1024 * 1024 * 1024)).toFixed(1)}B values`
  }

  const formatCoordinateValue = (value) => {
    if (typeof value === 'number') {
      return value.toPrecision(4)
    }
    return value
  }

  const varInfo = selectedVariable ? variables[selectedVariable] : null

  return (
    <section className="variable-selector">
      <h2>Variables ({variableNames.length})</h2>
      
      <div className="variable-list">
        {variableNames.map(name => {
          const info = variables[name]
          const totalSize = variableSizes[name] || 0
          const isCoordinate = coordinates[name] !== undefined
          const isGridVariable = metadata.gridVariables && metadata.gridVariables.has(name)
          
          return (
            <div 
              key={name}
              className={`variable-item ${selectedVariable === name ? 'selected' : ''} ${isCoordinate ? 'coordinate' : ''} ${isGridVariable ? 'grid-variable' : ''}`}
              onClick={() => onVariableSelect(name)}
            >
              <div className="variable-header">
                <div className="variable-name">
                  <strong>{name}</strong>
                  {isCoordinate && <span className="coord-badge">coordinate</span>}
                  {isGridVariable && <span className="grid-badge">grid variable</span>}
                </div>
                <div className="variable-meta">
                  <span className="type">{info.data_type}</span>
                  <span className="size">{formatBytes(totalSize)}</span>
                </div>
              </div>
              
              <div className="variable-details">
                {info.dimensions && info.dimensions.length > 0 ? (
                  <div className="dimensions">
                    <strong>Dimensions:</strong> {info.dimensions.map(d => `${d.name}[${d.size}]`).join(' × ')}
                  </div>
                ) : (
                  <div className="dimensions">
                    <strong>Type:</strong> Scalar variable
                  </div>
                )}
                
                {info.attributes && Object.keys(info.attributes).length > 0 && (
                  <div className="attributes">
                    {info.attributes.units && (
                      <span className="units"><strong>Units:</strong> {info.attributes.units}</span>
                    )}
                    {info.attributes.long_name && (
                      <span className="description"><strong>Description:</strong> {info.attributes.long_name}</span>
                    )}
                  </div>
                )}

                {coordinates[name] && (
                  <div className="coordinate-preview">
                    <strong>Sample values:</strong>
                    <div className="coord-values">
                      {coordinates[name].values.map((val, idx) => (
                        <span key={idx} className="coord-value">
                          [{idx}]: {formatCoordinateValue(val)}
                        </span>
                      ))}
                      {coordinates[name].size > coordinates[name].values.length && (
                        <span className="coord-ellipsis">... ({coordinates[name].size} total)</span>
                      )}
                    </div>
                  </div>
                )}
              </div>
            </div>
          )
        })}
      </div>

      {selectedVariable && varInfo && (
        <div className="index-selector">
          <h3>Data Selection for {selectedVariable}</h3>
          
          <div className="variable-summary">
            <div className="summary-item">
              <strong>Type:</strong> {varInfo.data_type}
            </div>
            <div className="summary-item">
              <strong>Total Size:</strong> {formatBytes(variableSizes[selectedVariable] || 0)}
            </div>
            {varInfo.attributes?.units && (
              <div className="summary-item">
                <strong>Units:</strong> {varInfo.attributes.units}
              </div>
            )}
          </div>

          {varInfo.dimensions && varInfo.dimensions.length > 0 ? (
            <>
              <div className="dimension-controls">
                <h4>Select Ranges:</h4>
                {varInfo.dimensions.map(dim => {
                  const coordInfo = coordinates[dim.name]
                  const currentRange = selectedIndices[dim.name] || { min: 0, max: 0 }
                  const rangeSize = Math.max(1, currentRange.max - currentRange.min + 1)
                  
                  return (
                    <div key={dim.name} className="dimension-control">
                      <div className="dimension-header">
                        <label className="dimension-label">
                          {dim.name} 
                          <span className="range-size">({rangeSize} element{rangeSize !== 1 ? 's' : ''})</span>
                        </label>
                        <span className="dimension-range">0 to {dim.size - 1}</span>
                      </div>
                      
                      <div className="range-inputs">
                        <div className="range-input-group">
                          <label className="range-label">Min:</label>
                          <input
                            type="number"
                            min="0"
                            max={Math.min(currentRange.max, dim.size - 1)}
                            value={currentRange.min}
                            onChange={(e) => handleRangeChange(dim.name, 'min', e.target.value)}
                            className="range-number"
                          />
                          <input
                            type="range"
                            min="0"
                            max={Math.min(currentRange.max, dim.size - 1)}
                            value={currentRange.min}
                            onChange={(e) => handleRangeChange(dim.name, 'min', e.target.value)}
                            className="range-slider"
                          />
                        </div>
                        
                        <div className="range-input-group">
                          <label className="range-label">Max:</label>
                          <input
                            type="number"
                            min={Math.max(currentRange.min, 0)}
                            max={dim.size - 1}
                            value={currentRange.max}
                            onChange={(e) => handleRangeChange(dim.name, 'max', e.target.value)}
                            className="range-number"
                          />
                          <input
                            type="range"
                            min={Math.max(currentRange.min, 0)}
                            max={dim.size - 1}
                            value={currentRange.max}
                            onChange={(e) => handleRangeChange(dim.name, 'max', e.target.value)}
                            className="range-slider"
                          />
                        </div>
                      </div>

                      {coordInfo && coordInfo.values.length > 0 && (
                        <div className="coordinate-info">
                          <span className="coord-label">Range values:</span>
                          <div className="coord-range">
                            <span className="coord-value">
                              From: {currentRange.min < coordInfo.values.length 
                                ? formatCoordinateValue(coordInfo.values[currentRange.min])
                                : `~${formatCoordinateValue(coordInfo.values[0])}`
                              }
                            </span>
                            <span className="coord-value">
                              To: {currentRange.max < coordInfo.values.length 
                                ? formatCoordinateValue(coordInfo.values[currentRange.max])
                                : `~${formatCoordinateValue(coordInfo.values[coordInfo.values.length - 1])}`
                              }
                            </span>
                            {coordInfo.attributes?.units && (
                              <span className="coord-units">{coordInfo.attributes.units}</span>
                            )}
                          </div>
                        </div>
                      )}
                    </div>
                  )
                })}
              </div>
              
              <div className="fetch-controls">
                <button 
                  onClick={onFetchData}
                  disabled={loading}
                  className="fetch-button"
                >
                  {loading ? 'Fetching...' : 'Fetch Range Data'}
                </button>
                <div className="fetch-info">
                  <span className="data-size">Will fetch {Object.values(selectedIndices).reduce((total, range) => total * Math.max(1, range.max - range.min + 1), 1)} values</span>
                </div>
              </div>
            </>
          ) : (
            <div className="scalar-fetch">
              <p>This is a scalar variable with a single value.</p>
              <button 
                onClick={onFetchData}
                disabled={loading}
                className="fetch-button"
              >
                {loading ? 'Fetching...' : 'Fetch Value'}
              </button>
            </div>
          )}
        </div>
      )}
    </section>
  )
}

export default VariableSelector