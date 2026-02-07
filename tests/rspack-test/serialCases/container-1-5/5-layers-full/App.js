import React, { layeredComponentsReact } from 'react';
import ComponentA from './ComponentA';
export default function App() {
  return `App rendered with React version: [${React()}] with layer [${layeredComponentsReact()}] ${ComponentA()}`;
}
