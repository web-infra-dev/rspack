import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { answer } from 'lib3';
console.log('answer:', answer());

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
