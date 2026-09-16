import path from 'node:path';
import fs from 'node:fs/promises';

const libAIndex = path.resolve(import.meta.dirname, './lib/a/index');
let index = 0;

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.resolve(import.meta.dirname, './file.js')],
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tapPromise('TestPlugin', async function () {
          index++;
          if (index === 1) {
            await fs.writeFile(libAIndex, String(index));
          }
        });
      },
    },
  ],
};
