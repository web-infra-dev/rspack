import { events } from './state.js';

export const value = await Promise.resolve(42);
events.push('async');
