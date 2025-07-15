// Main Three.js-like entry point with many dependencies

import * as Cameras from './cameras/index.js';
import * as Controls from './controls/index.js';
import * as Core from './core/index.js';
import * as Geometries from './geometries/index.js';
import * as Lights from './lights/index.js';
import * as Loaders from './loaders/index.js';
import * as Materials from './materials/index.js';
import * as Math from './math/index.js';
import * as Objects from './objects/index.js';
import * as Renderers from './renderers/index.js';
import * as Scenes from './scenes/index.js';
import * as Utils from './utils/index.js';

// Simulate complex dependency tree
export {
  Core,
  Math,
  Geometries,
  Materials,
  Objects,
  Scenes,
  Lights,
  Cameras,
  Renderers,
  Loaders,
  Controls,
  Utils
};

export const VERSION = '1.0.0';
