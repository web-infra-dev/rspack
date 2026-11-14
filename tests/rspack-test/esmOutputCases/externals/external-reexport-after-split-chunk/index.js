import './setup.mjs';
export { value } from './ext.mjs';
import './side.mjs';

expect(globalThis.__order).toEqual(['setup', 'ext', 'side']);
