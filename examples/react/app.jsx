import './base.css';
import React from 'react';
import ReactDOM from 'react-dom';
const Button = React.lazy(() => import('./button'));
const App = () => {
  return (
    <React.Suspense fallback={<div>loading...</div>}>
      <div>hello world</div>
      <Button></Button>
    </React.Suspense>
  );
};
ReactDOM.render(<App />, document.getElementById('root'));
