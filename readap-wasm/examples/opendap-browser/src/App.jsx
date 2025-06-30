import { useState, useEffect } from 'react'
import './App.css'
import DatasetBrowser from './components/DatasetBrowser'

function App() {
  const [error, setError] = useState(null)

  return (
    <div className="App">
      <header>
        <h1>OpenDAP Dataset Browser</h1>
        <p>Browse and visualize OpenDAP datasets using @mattnucc/readap</p>
      </header>
      
      {error && (
        <div className="error">
          <strong>Error:</strong> {error}
        </div>
      )}
      
      <DatasetBrowser onError={setError} />
    </div>
  )
}

export default App
