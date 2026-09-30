import { defineConfig, definePlugin } from '@rspack/cli';
import type { FileSystemCacheOptions } from '@rspack/core';
import path from 'node:path';
import fs from 'node:fs/promises';

const cacheDir = path.join(import.meta.dirname, 'node_modules/.cache/test');
const cacheLocation = path.join(cacheDir, 'test-cache');

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    name: 'test-cache',
    storage: {
      type: 'filesystem',
      directory: cacheDir,
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tapPromise('Test Plugin', async function () {
          const cache = compiler.options.cache as FileSystemCacheOptions;
          expect(cache.name).toBe('test-cache');
          expect(cache.cacheDirectory).toBe(cacheDir);
          expect(cache.cacheLocation).toBe(cacheLocation);
          const stat = await fs.stat(cacheLocation);
          expect(stat.isDirectory()).toBeTruthy();
        });
      },
    }),
  ],
});
