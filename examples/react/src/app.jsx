import './base.css';
import React from 'react';
import ReactDOM from 'react-dom';
const Button = React.lazy(() => import('../src/button'));
import LogoUrl from './logo.svg';
import Logo from './logo.svg?svgr';
console.log("LogoUrl", LogoUrl)
console.log("Logo", Logo)

const App = () => {
  return (
    <React.Suspense fallback={<div>loading...</div>}>
      <div>hello world</div>
      <Button></Button>
      <img src={LogoUrl} alt='logo' />
      <Logo />
    </React.Suspense>
  );
};
ReactDOM.render(<App />, document.getElementById('root'));
