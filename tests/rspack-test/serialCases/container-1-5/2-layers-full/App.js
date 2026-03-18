import React, { layeredComponentsReact } from 'react';
import ComponentA from 'containerA/ComponentA';
import ComponentB from 'containerB/ComponentB';
import ComponentALayers from 'containerB/ComponentALayers';
import LocalComponentB from './ComponentB';
import LocalComponentALayers from './ComponentALayers';

export default () => {
  return `App rendered with [${React()}] ${layeredComponentsReact()} and ${ComponentALayers()} and ${LocalComponentALayers()} and [${ComponentA()}] and [${ComponentB()}] and ${LocalComponentB()}`;
};
