import './base.css';
import React from 'react';
import ReactDOM from 'react-dom';
const Button = React.lazy(import('./button'));
const App = () => {
  return (
    <>
      <div>hello world</div>
      <Button></Button>
    </>
  );
};
ReactDOM.render(<App />, document.getElementById('root'));
