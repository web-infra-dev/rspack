export { yv } from './y.js';
import './x.js';

expect(globalThis.__order).toEqual(['y', 'x']);
