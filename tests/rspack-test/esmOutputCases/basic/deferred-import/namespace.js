import { events } from './state.js';
import { value as shared } from './shared.js';

events.push('namespace');
export const value = shared + 1;
