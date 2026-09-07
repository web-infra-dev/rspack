const path = require('path');
const fs = require('fs/promises');

const cacheDir = path.join(__dirname, 'node_modules/.cache/test');
const cacheLocation = path.join(cacheDir, 'test-cache');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
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
          expect(compiler.options.cache.storage.directory).toBe(cacheDir);
          expect(compiler.options.cache.storage.location).toBe(cacheLocation);
          const stat = await fs.stat(cacheLocation);
          expect(stat.isDirectory()).toBeTruthy();
        });
      },
    },
  ],
};
