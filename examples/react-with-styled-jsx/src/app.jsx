import React from 'react';

export const App = () => {
  return (
    <React.Suspense fallback={<div>loading...</div>}>
      <div>
        <div>
          <h1>I am Red</h1>
          <style jsx>{`
            h1 {
              color: red;
            }
          `}</style>
        </div>
        <div>
          <h1>I am NOT Red</h1>
        </div>
      </div>

    </React.Suspense>
  );
};
