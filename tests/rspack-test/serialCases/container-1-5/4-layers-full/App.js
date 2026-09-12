import React from 'react';
import ComponentA from 'containerA/ComponentA';

export default function App() {
  return `App rendered with React version: [${React()}]\nand remote component: [${ComponentA()}]`;
}
