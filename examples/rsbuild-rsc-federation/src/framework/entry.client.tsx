import React from 'react';
import { createRoot } from 'react-dom/client';

const host = document.createElement('div');
document.body.appendChild(host);

createRoot(host).render(
  <React.StrictMode>
    <div data-client-entry="ready">client entry ready</div>
  </React.StrictMode>,
);
