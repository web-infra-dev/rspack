import { createRequire } from 'node:module';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

const require = createRequire(import.meta.url);
const coreDir = path.dirname(require.resolve('@rspack/core/package.json'));

export default defineConfig({
  target: 'node',
  plugins: [
    new DefinePlugin({
      TINYPOOL_FILENAME: JSON.stringify(
        path.join(coreDir, 'compiled/tinypool/dist/index.js'),
      ),
      WORKER_FILENAME: JSON.stringify(
        path.join(import.meta.dirname, 'worker.mjs'),
      ),
    }),
  ],
});
