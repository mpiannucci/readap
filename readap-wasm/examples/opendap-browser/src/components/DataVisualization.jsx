function DataVisualization({ data, metadata }) {
  if (!data) return null

  const formatValue = (value) => {
    if (typeof value === 'number') {
      return Number.isInteger(value) ? value.toString() : value.toPrecision(6)
    }
    return value
  }

  const renderDataTable = () => {
    if (data.data.length <= 20) {
      // For small datasets, show all values
      return (
        <table className="data-table">
          <thead>
            <tr>
              <th>Index</th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            {data.data.map((value, index) => (
              <tr key={index}>
                <td>{index}</td>
                <td>{formatValue(value)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )
    } else {
      // For larger datasets, show first/last few values
      const first10 = data.data.slice(0, 10)
      const last10 = data.data.slice(-10)
      
      return (
        <div className="data-summary">
          <h4>First 10 values:</h4>
          <table className="data-table">
            <thead>
              <tr>
                <th>Index</th>
                <th>Value</th>
              </tr>
            </thead>
            <tbody>
              {first10.map((value, index) => (
                <tr key={index}>
                  <td>{index}</td>
                  <td>{formatValue(value)}</td>
                </tr>
              ))}
            </tbody>
          </table>
          
          <div className="ellipsis">... {data.data.length - 20} more values ...</div>
          
          <h4>Last 10 values:</h4>
          <table className="data-table">
            <thead>
              <tr>
                <th>Index</th>
                <th>Value</th>
              </tr>
            </thead>
            <tbody>
              {last10.map((value, index) => (
                <tr key={index}>
                  <td>{data.data.length - 10 + index}</td>
                  <td>{formatValue(value)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )
    }
  }

  const renderStatistics = () => {
    if (data.data.length === 0) return null

    const numericData = data.data.filter(v => typeof v === 'number' && !isNaN(v))
    
    if (numericData.length === 0) return null

    const min = Math.min(...numericData)
    const max = Math.max(...numericData)
    const mean = numericData.reduce((sum, v) => sum + v, 0) / numericData.length
    
    return (
      <div className="statistics">
        <h4>Statistics</h4>
        <div className="stat-grid">
          <div className="stat">
            <label>Count:</label>
            <span>{data.data.length}</span>
          </div>
          <div className="stat">
            <label>Min:</label>
            <span>{formatValue(min)}</span>
          </div>
          <div className="stat">
            <label>Max:</label>
            <span>{formatValue(max)}</span>
          </div>
          <div className="stat">
            <label>Mean:</label>
            <span>{formatValue(mean)}</span>
          </div>
        </div>
      </div>
    )
  }

  const renderPixelGrid = () => {
    if (data.data.length === 0) return null

    const numericData = data.data.filter(v => typeof v === 'number' && !isNaN(v))
    if (numericData.length === 0) return null

    const min = Math.min(...numericData)
    const max = Math.max(...numericData)
    const range = max - min

    // Try to infer grid dimensions from data length
    let gridWidth, gridHeight
    const totalElements = data.data.length

    if (totalElements === 1) {
      // Single value - show as 1x1 grid
      gridWidth = gridHeight = 1
    } else if (totalElements <= 100) {
      // For small arrays, try to make a square-ish grid
      gridWidth = Math.ceil(Math.sqrt(totalElements))
      gridHeight = Math.ceil(totalElements / gridWidth)
    } else if (totalElements <= 10000) {
      // For medium arrays, limit to reasonable display size
      const maxDim = 100
      gridWidth = Math.min(maxDim, Math.ceil(Math.sqrt(totalElements)))
      gridHeight = Math.min(maxDim, Math.ceil(totalElements / gridWidth))
    } else {
      // For very large arrays, sample and show warning
      gridWidth = gridHeight = 50
    }

    const getColorForValue = (value) => {
      if (typeof value !== 'number' || isNaN(value)) {
        return '#ccc' // Gray for invalid values
      }
      
      if (range === 0) {
        return '#3498db' // Blue for constant values
      }
      
      // Map value to hue (blue to red)
      const normalized = (value - min) / range
      const hue = (1 - normalized) * 240 // 240 = blue, 0 = red
      return `hsl(${hue}, 70%, 50%)`
    }

    const getIndicesForPosition = (flatIndex) => {
      // For 1D data, just return the index
      if (gridHeight === 1) {
        return `[${flatIndex}]`
      }
      
      // For 2D layout, calculate row/col
      const row = Math.floor(flatIndex / gridWidth)
      const col = flatIndex % gridWidth
      return `[${row}, ${col}]`
    }

    return (
      <div className="pixel-grid">
        <h4>Data Visualization ({gridWidth} × {gridHeight} grid)</h4>
        {totalElements > gridWidth * gridHeight && (
          <p className="grid-warning">
            Showing first {gridWidth * gridHeight} of {totalElements} values
          </p>
        )}
        <div className="color-scale">
          <span className="scale-label">Min: {formatValue(min)}</span>
          <div className="scale-gradient" style={{
            background: `linear-gradient(to right, hsl(240, 70%, 50%), hsl(120, 70%, 50%), hsl(0, 70%, 50%))`
          }}></div>
          <span className="scale-label">Max: {formatValue(max)}</span>
        </div>
        <div 
          className="grid-container"
          style={{
            gridTemplateColumns: `repeat(${gridWidth}, 1fr)`,
            gridTemplateRows: `repeat(${gridHeight}, 1fr)`
          }}
        >
          {data.data.slice(0, gridWidth * gridHeight).map((value, index) => (
            <div
              key={index}
              className="grid-pixel"
              style={{ backgroundColor: getColorForValue(value) }}
              title={`Indices ${getIndicesForPosition(index)}: ${formatValue(value)}`}
              onMouseEnter={(e) => {
                const tooltip = document.getElementById('pixel-tooltip')
                if (tooltip) {
                  tooltip.style.display = 'block'
                  tooltip.style.left = e.pageX + 10 + 'px'
                  tooltip.style.top = e.pageY - 10 + 'px'
                  tooltip.innerHTML = `
                    <strong>Indices:</strong> ${getIndicesForPosition(index)}<br/>
                    <strong>Value:</strong> ${formatValue(value)}
                  `
                }
              }}
              onMouseLeave={() => {
                const tooltip = document.getElementById('pixel-tooltip')
                if (tooltip) {
                  tooltip.style.display = 'none'
                }
              }}
              onMouseMove={(e) => {
                const tooltip = document.getElementById('pixel-tooltip')
                if (tooltip) {
                  tooltip.style.left = e.pageX + 10 + 'px'
                  tooltip.style.top = e.pageY - 10 + 'px'
                }
              }}
            />
          ))}
        </div>
        <div id="pixel-tooltip" className="pixel-tooltip"></div>
      </div>
    )
  }

  return (
    <section className="data-visualization">
      <h2>Data Visualization</h2>
      
      <div className="data-info">
        <h3>{data.variable}</h3>
        <p>Data type: {metadata.variables[data.variable]?.data_type}</p>
        <p>Values fetched: {data.data.length}</p>
        {data.constraint && (
          <p>Constraint used: <code>{data.constraint}</code></p>
        )}
        
        {data.attributes && Object.keys(data.attributes).length > 0 && (
          <div className="attributes">
            <h4>Attributes:</h4>
            <ul>
              {Object.entries(data.attributes).map(([key, value]) => (
                <li key={key}>
                  <strong>{key}:</strong> {value}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {renderStatistics()}
      {renderPixelGrid()}
      {renderDataTable()}
    </section>
  )
}

export default DataVisualization