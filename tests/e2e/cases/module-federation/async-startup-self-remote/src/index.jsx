import React, { Suspense, useState } from 'react';
import ReactDOM from 'react-dom/client';

const RemoteComponent = React.lazy(() => import('remote/RemoteComponent'));

function App() {
  const [showRemote, setShowRemote] = useState(false);
  return (
    <div>
      <button type="button" onClick={() => setShowRemote(true)}>
        Load Remote
      </button>
      {showRemote ? (
        <Suspense fallback={<div>Loading remote...</div>}>
          <RemoteComponent />
        </Suspense>
      ) : null}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
