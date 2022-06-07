import * as React from 'react';
import LogoUrl from './logo.svg';
import Logo from './logo.svg';

console.log('LogoUrl', LogoUrl);
console.log('Logo', Logo);
const App = () => {
  return (
    <React.Suspense fallback={<div>loading...</div>}>
      <img style={{ width: '40px', height: '40px' }} src={LogoUrl} alt='logo' />
      <Logo width={'40px'} height={'40px'} />
    </React.Suspense>
  );
};
export default App;
