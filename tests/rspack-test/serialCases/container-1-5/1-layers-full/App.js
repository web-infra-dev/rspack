import React, { layeredComponentsReact } from 'react';
import ComponentA from 'containerA/ComponentA';
import ComponentALayers from 'containerA/ComponentALayers';
import LocalComponentALayers from './ComponentALayers';
import LocalComponentA from './ComponentA';

export default () => {
  return `App rendered with [${React()}] ${layeredComponentsReact()}, ${LocalComponentALayers()}, ${LocalComponentA()}, [${ComponentA()}] and [${ComponentALayers()}]`;
};
