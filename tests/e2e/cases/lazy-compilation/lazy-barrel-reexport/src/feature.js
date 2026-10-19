import { createStore } from './lib/index.js';
import { extra } from './default-barrel.js';

export const value = `${createStore('feature').state}:${extra()}`;
