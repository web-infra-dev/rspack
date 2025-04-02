import React, { useState, useEffect } from 'react';
// Import a remote React component to demonstrate remote import capabilities
import RemoteComponent from 'https://esm.sh/lodash@4.17.21/es2022/lodash.mjs';

// Simple React component to demonstrate React Refresh functionality
function App({ message, getMessage }) {
  const [count, setCount] = useState(0);
  const [time, setTime] = useState(new Date().toLocaleTimeString());
  
  // This effect will be preserved during Hot Module Replacement
  useEffect(() => {
    const timer = setInterval(() => {
      setTime(new Date().toLocaleTimeString());
    }, 1000);
    
    return () => {
      clearInterval(timer);
    };
  }, []);

  return (
    <div className="app-container" style={{ 
      fontFamily: 'system-ui, sans-serif',
      maxWidth: '800px',
      margin: '0 auto',
      padding: '20px',
      backgroundColor: '#f5f5f5',
      borderRadius: '8px',
      boxShadow: '0 2px 10px rgba(0,0,0,0.1)'
    }}>
      <h1>React Refresh with HTTP Imports</h1>
      <p>Current time: <strong>{time}</strong></p>
      <p>Message from imported module: <strong>{message}</strong></p>
      <p>Function result: <strong>{getMessage()}</strong></p>
      
      <div className="counter" style={{ 
        marginTop: '20px',
        padding: '15px',
        backgroundColor: '#fff',
        borderRadius: '6px'
      }}>
        <p>Count: <strong>{count}</strong></p>
        <button 
          onClick={() => setCount(count + 1)}
          style={{
            backgroundColor: '#0070f3',
            color: 'white',
            border: 'none',
            padding: '8px 16px',
            borderRadius: '4px',
            cursor: 'pointer'
          }}
        >
          Increment
        </button>
      </div>
      
      {/* Render the remote component */}
      <div style={{ marginTop: '20px' }}>
        <RemoteComponent label="This is loaded from a remote URL" />
      </div>
      
      <p className="note" style={{ 
        marginTop: '20px',
        backgroundColor: '#fffde7',
        padding: '10px',
        borderLeft: '4px solid #ffd600',
        borderRadius: '4px'
      }}>
        <strong>HOW TO TEST:</strong> Edit this file and the changes should be applied without losing the counter state.
        Try changing colors, text, or adding new elements to see Fast Refresh in action.
      </p>
    </div>
  );
}

export default App; 