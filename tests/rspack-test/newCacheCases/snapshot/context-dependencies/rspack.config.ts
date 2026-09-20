import { defineConfig, definePlugin } from '@rspack/cli';

import path from 'node:path';
import fs from 'node:fs/promises';

const libAIndex = path.resolve(import.meta.dirname, './lib/a/index');
let index = 0;

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.join(import.meta.dirname, 'immutable')],
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tapPromise('TestPlugin', async function () {
          index++;
          if (index === 1) {
            await fs.writeFile(libAIndex, String(index));
          }
        });
      },
    }),
  ],
});
