import React, { layeredComponentsReact } from 'react';

export default function ComponentA() {
  return `ComponentA rendered with React version: [${React()}] with layer [${layeredComponentsReact()}]`;
}
