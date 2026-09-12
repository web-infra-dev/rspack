import React, { Suspense, useState } from 'react';
import { createRoot } from 'react-dom/client';

const Lazy = React.lazy(() => import('./Lazy.jsx'));

function App() {
  const [showLazy, setShowLazy] = useState(false);

  return showLazy ? (
    <Suspense fallback="loading">
      <Lazy />
    </Suspense>
  ) : (
    <button type="button" onClick={() => setShowLazy(true)}>
      load lazy module
    </button>
  );
}

createRoot(document.getElementById('root')).render(<App />);
