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

  const renderSimpleChart = () => {
    if (data.data.length === 0 || data.data.length > 100) return null

    const numericData = data.data.filter(v => typeof v === 'number' && !isNaN(v))
    if (numericData.length === 0) return null

    const min = Math.min(...numericData)
    const max = Math.max(...numericData)
    const range = max - min

    if (range === 0) return null

    return (
      <div className="simple-chart">
        <h4>Data Visualization</h4>
        <div className="chart-container">
          {data.data.map((value, index) => {
            if (typeof value !== 'number' || isNaN(value)) return null
            
            const height = range > 0 ? ((value - min) / range) * 100 : 50
            
            return (
              <div 
                key={index}
                className="bar"
                style={{ 
                  height: `${height}%`,
                  left: `${(index / data.data.length) * 100}%`,
                  width: `${100 / data.data.length}%`
                }}
                title={`Index ${index}: ${formatValue(value)}`}
              />
            )
          })}
        </div>
      </div>
    )
  }

  return (
    <section className="data-visualization">
      <h2>Data Visualization</h2>
      
      <div className="data-info">
        <h3>{data.variable}</h3>
        <p>Data type: {metadata.variables[data.variable]?.data_type}</p>
        <p>Total values: {data.data.length}</p>
        
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
      {renderSimpleChart()}
      {renderDataTable()}
    </section>
  )
}

export default DataVisualization