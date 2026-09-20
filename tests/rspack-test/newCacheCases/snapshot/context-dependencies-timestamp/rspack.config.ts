import { defineConfig, definePlugin } from '@rspack/cli';

import path from 'node:path';
import fs from 'node:fs/promises';

const libAIndex = path.resolve(import.meta.dirname, './lib/a/index');
// Both timestamps must precede the builds so safe-time checks allow cache hits.
const initialTime = new Date('2000-01-01T00:00:00Z');
const modifiedTime = new Date('2000-01-02T00:00:00Z');
let index = 0;

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      module: { timestamp: true },
      immutablePaths: [path.join(import.meta.dirname, 'immutable')],
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.beforeRun.tapPromise('TestPlugin', async function () {
          if (index === 0) {
            await fs.utimes(libAIndex, initialTime, initialTime);
          }
        });
        compiler.hooks.done.tapPromise('TestPlugin', async function () {
          index++;
          if (index === 1) {
            // Change only a context member's timestamp, keeping its contents stable.
            await fs.utimes(libAIndex, modifiedTime, modifiedTime);
          }
        });
      },
    }),
  ],
});
