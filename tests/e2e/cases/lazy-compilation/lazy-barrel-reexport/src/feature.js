import { createStore } from './lib/index.js';
import { createStore as createNamedStore } from './lib/named.js';

export const value = `${createStore('feature').state},${createNamedStore('named-feature').state}`;
