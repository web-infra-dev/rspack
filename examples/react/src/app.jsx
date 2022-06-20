import React from 'react'
import ReactDOM from 'react-dom'
import './base.css'
// import Dark from './dark.svg'
// import Light from './light.svg'
// import LogoUrl from './logo.svg'
// import Logo from './logo.svg'
// const Button = React.lazy(() => import('../src/button'))

// console.log('LogoUrl', LogoUrl)
// console.log('Logo', Logo)
const App = () => {
  return (
    <React.Suspense fallback={<div>loading...</div>}>
      <div>hello world</div>
      {/* <Button></Button> */}

      {/* <img style={{ width: '40px', height: '40px' }} src={LogoUrl} alt="logo" /> */}
      {
        /* <Logo width={'40px'} height={'40px'} />
      <Light width={'40px'} height={'40px'} />
      <Dark width={'40px'} height={'40px'} /> */
      }
    </React.Suspense>
  )
}
ReactDOM.render(<App />, document.getElementById('root'))
