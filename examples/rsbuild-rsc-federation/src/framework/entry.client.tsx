import React from 'react';
import { createRoot } from 'react-dom/client';
import { createFromFetch } from 'react-server-dom-rspack/client.browser';

void createFromFetch;

function InteractiveClientDemo() {
  const [count, setCount] = React.useState(0);

  return (
    <main data-testid="app-ready">
      <h1 data-testid="status-text">client entry ready</h1>
      <p data-testid="component-rendered">InteractiveClientDemo</p>
      <button
        data-testid="increment-button"
        type="button"
        onClick={() => setCount((value) => value + 1)}
      >
        increment
      </button>
      <output data-testid="counter-value">{count}</output>
    </main>
  );
}

const host = document.createElement('div');
document.body.appendChild(host);

createRoot(host).render(
  <React.StrictMode>
    <InteractiveClientDemo />
  </React.StrictMode>,
);
