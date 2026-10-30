import { aSide } from './a-side.js';
import shared from './shared.cjs';

export const a = aSide + shared.value;
export const loadC = () => import('./c.js');
