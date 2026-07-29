import React from 'react';
import ComponentA from 'containerA/ComponentA';
import LocalComponentA from './ComponentA';

export default function App() {
  return `App rendered with React version: [${React()}]\nand remote component: [${ComponentA()}]\n and local component: [${LocalComponentA()}]`;
}
