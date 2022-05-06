import './base.css';
import React from 'react';
import ReactDOM from 'react-dom';
import Button from './button';
const App = () => {
  return (
    <>
      <div>hello world</div>
      <Button></Button>
    </>
  );
};
ReactDOM.render(<App />, document.getElementById('root'));
