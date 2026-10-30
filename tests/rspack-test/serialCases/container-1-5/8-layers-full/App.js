import React, { layeredComponentsReact } from 'react';
import ComponentA from 'containerA/ComponentA';
import RemoteApp from 'containerA/App';
import LocalComponentA from './ComponentA';

export default function App() {
  return `App (no layer) rendered with React version: [${React()}] with non-layered React value: [${layeredComponentsReact()}]
Local Component: ${LocalComponentA()}
Remote Component from container7: ${ComponentA()}
Remote App from container7: ${RemoteApp()}`;
}
