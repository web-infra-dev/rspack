// @ts-expect-error: for those who need to create extra memfs for inputFileSystem
import { memfs as __memfs } from '@napi-rs/wasm-runtime/fs';
// @ts-expect-error: shimmed with rspack.wasi-browser.js
import { __fs, __volume } from '@rspack/binding';

import type { IFs, memfs as Memfs, Volume } from 'memfs';
type NodeFs = typeof import('node:fs');
type BrowserFsExports = Pick<
  NodeFs,
  'readFileSync' | 'readdirSync' | 'lstat' | 'existsSync' | 'readdir' | 'watch'
>;

export const fs: IFs = __fs;
export const volume: Volume = __volume;
export const memfs: typeof Memfs = __memfs;

const { readFileSync, readdirSync, lstat, existsSync, readdir, watch } =
  fs as unknown as BrowserFsExports;

export default fs;
export { existsSync, lstat, readdir, readdirSync, readFileSync, watch };
