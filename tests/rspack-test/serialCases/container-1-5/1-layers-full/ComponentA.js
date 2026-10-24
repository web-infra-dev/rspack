import React, { layeredComponentsReact } from 'react';

export default () => {
  return `ComponentA rendered with [${React()}]${layeredComponentsReact()}`;
};
