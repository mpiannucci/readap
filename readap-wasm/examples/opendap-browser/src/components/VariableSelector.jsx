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

  // Reset indices when variable changes
  useEffect(() => {
    if (selectedVariable && variables[selectedVariable]) {
      const varInfo = variables[selectedVariable]
      const defaultIndices = {}
      
      if (varInfo.dimensions) {
        varInfo.dimensions.forEach(dim => {
          defaultIndices[dim.name] = 0 // Default to first index
        })
      }
      
      onIndicesChange(defaultIndices)
    }
  }, [selectedVariable, variables, onIndicesChange])

  const handleIndexChange = (dimName, value) => {
    const numValue = parseInt(value, 10)
    if (!isNaN(numValue)) {
      onIndicesChange(prev => ({
        ...prev,
        [dimName]: numValue
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
          
          return (
            <div 
              key={name}
              className={`variable-item ${selectedVariable === name ? 'selected' : ''} ${isCoordinate ? 'coordinate' : ''}`}
              onClick={() => onVariableSelect(name)}
            >
              <div className="variable-header">
                <div className="variable-name">
                  <strong>{name}</strong>
                  {isCoordinate && <span className="coord-badge">coordinate</span>}
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
                <h4>Select Indices:</h4>
                {varInfo.dimensions.map(dim => {
                  const coordInfo = coordinates[dim.name]
                  const currentIndex = selectedIndices[dim.name] || 0
                  
                  return (
                    <div key={dim.name} className="dimension-control">
                      <div className="dimension-header">
                        <label className="dimension-label">{dim.name}</label>
                        <span className="dimension-range">0 to {dim.size - 1}</span>
                      </div>
                      
                      <div className="dimension-input">
                        <input
                          type="range"
                          min="0"
                          max={dim.size - 1}
                          value={currentIndex}
                          onChange={(e) => handleIndexChange(dim.name, e.target.value)}
                          className="dimension-slider"
                        />
                        <input
                          type="number"
                          min="0"
                          max={dim.size - 1}
                          value={currentIndex}
                          onChange={(e) => handleIndexChange(dim.name, e.target.value)}
                          className="dimension-number"
                        />
                      </div>

                      {coordInfo && coordInfo.values.length > 0 && (
                        <div className="coordinate-info">
                          <span className="coord-label">Coordinate value:</span>
                          <span className="coord-current">
                            {currentIndex < coordInfo.values.length 
                              ? formatCoordinateValue(coordInfo.values[currentIndex])
                              : `~${formatCoordinateValue(coordInfo.values[coordInfo.values.length - 1])}`
                            }
                            {coordInfo.attributes?.units && ` ${coordInfo.attributes.units}`}
                          </span>
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
                  {loading ? 'Fetching...' : 'Fetch Single Value'}
                </button>
                <div className="fetch-info">
                  <span className="data-size">Will fetch 1 value at selected indices</span>
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