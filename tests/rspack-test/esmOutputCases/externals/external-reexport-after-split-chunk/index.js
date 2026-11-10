import './setup.mjs';
export { value } from './ext.mjs';
import './side.mjs';

expect(globalThis.__order).toEqual(['side', 'setup', 'ext']);
