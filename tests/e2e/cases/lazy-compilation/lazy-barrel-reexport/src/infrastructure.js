import { create } from './lib/index.js';
import { create as createNamed } from './lib/named.js';

export const infra = `${create('star').state},${createNamed('named').state}`;
