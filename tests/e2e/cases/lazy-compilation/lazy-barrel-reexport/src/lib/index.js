import { createStore } from './vanilla.js';

export * from './vanilla.js';
export const create = (init) => createStore(init);
