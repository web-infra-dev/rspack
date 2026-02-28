export * from '../index';
export { BrowserHttpImportEsmPlugin } from './BrowserHttpImportEsmPlugin';
export { BrowserRequirePlugin } from './BrowserRequirePlugin';

import { fs, memfs, volume } from './fs';
export const builtinMemFs = {
  fs,
  volume,
  memfs,
};
