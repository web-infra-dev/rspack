import React, { layeredComponentsReact } from 'react';

export default function ComponentA() {
  return `LocalComponentA (in react-layer) rendered with React version: [${React()}], layered React value: [${layeredComponentsReact()}]`;
}
