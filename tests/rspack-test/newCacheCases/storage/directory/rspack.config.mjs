import path from 'node:path';
import fs from 'node:fs/promises';

const cacheDir = path.join(import.meta.dirname, 'node_modules/.cache/test');
const cacheLocation = path.join(cacheDir, 'test-cache');

/** @type {import("@rspack/core").Configuration} */
export default {
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
    {
      apply(compiler) {
        compiler.hooks.done.tapPromise('Test Plugin', async function () {
          expect(compiler.options.cache.name).toBe('test-cache');
          expect(compiler.options.cache.cacheDirectory).toBe(cacheDir);
          expect(compiler.options.cache.cacheLocation).toBe(cacheLocation);
          const stat = await fs.stat(cacheLocation);
          expect(stat.isDirectory()).toBeTruthy();
        });
      },
    },
  ],
};
