import * as fs from './lib.js';

export { fs };
export default fs;

import pkg from './small.json' with { type: 'json' };

export const value = Object.assign(Object.create(fs), { _version: pkg.version });
