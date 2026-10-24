import React, { layeredComponentsReact } from 'react';
import ComponentA from 'containerA/ComponentA';
import ComponentB from 'containerB/ComponentB';

export default () => {
  return `LocalComponentC rendered with [${React()}] ${layeredComponentsReact()} and [${ComponentA()}] and [${ComponentB()}]`;
};
