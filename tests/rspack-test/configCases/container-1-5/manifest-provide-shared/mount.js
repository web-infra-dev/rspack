import { value } from './static.js';
import './style.css';

export const mount = () => value;
export const load = () => import('./lazy.js');
