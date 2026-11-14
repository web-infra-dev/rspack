export { value } from './first.mjs';
import './second.mjs';

export const load = () => import('./unused.mjs');
