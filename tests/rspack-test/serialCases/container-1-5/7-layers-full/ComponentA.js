import React, { layeredComponentsReact } from 'react';

export default function ComponentA() {
  return `ComponentA (in react-layer) rendered with React version: [${React()}] with layered React value: [${layeredComponentsReact()}]`;
}
