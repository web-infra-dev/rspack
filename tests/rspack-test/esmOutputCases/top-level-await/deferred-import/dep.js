import './sync.js';
import { value as asyncValue } from './async.js';
import { events } from './state.js';

events.push('dep');
export const value = asyncValue;
