import React, { layeredComponentsReact } from 'react';
import ComponentA from './ComponentA';
export default function App() {
  return `App (no layer) rendered with React version: [${React()}] with non-layered React value: [${layeredComponentsReact()}] and imported: ${ComponentA()}`;
}
