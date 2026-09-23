import { events } from './state.js';
import { value as shared } from './shared.js';
import defer * as nested from './nested.js';

events.push('dep');
export const value = shared;
export const readNested = () => nested.value;
