import './globals.js';
export { default as shared } from './shared.cjs';
export const load = () => import('./async.cjs');
