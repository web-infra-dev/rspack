export { default as shared } from './shared.cjs';
import { __rspack_shared_modules__, __rspack_register_modules__ } from './marker.js';
export const markers = [__rspack_shared_modules__, __rspack_register_modules__];
export const load = () => import('./async.cjs');
