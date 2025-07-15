import { answer } from 'lib3';
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

console.log('answer:', answer());

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
