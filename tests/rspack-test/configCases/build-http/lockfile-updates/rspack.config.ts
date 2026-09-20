import { defineConfig, definePlugin } from '@rspack/cli';

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const lockfileLocation = path.join(import.meta.dirname, 'rspack.lock');
export default defineConfig({
  target: 'web',
  experiments: {
    buildHttp: {
      allowedUris: ['http://updates.example/'],
      frozen: false,
      cacheLocation: false,
      lockfileLocation,
      httpClient: async (url) => ({
        status: 200,
        headers: { 'content-type': 'application/javascript' },
        body: Buffer.from(
          `export default ${Number(new URL(url).pathname.slice(1, -3))};`,
        ),
      }),
    },
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.done.tap('CheckHttpLockfileUpdates', () => {
        const lockfile = JSON.parse(fs.readFileSync(lockfileLocation, 'utf8'));
        assert.deepEqual(
          Object.keys(lockfile.entries).sort(),
          Array.from({ length: 6 }, (_, i) => `http://updates.example/${i}.js`),
        );
      });
    }),
  ],
});
