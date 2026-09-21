import assert from 'node:assert/strict';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
    'worker-source': './worker.js',
  },
  externalsType: 'modern-module',
  externals: [
    ({ request, contextInfo }, callback) => {
      assert(contextInfo);
      if (contextInfo.issuer && request === './worker.js') {
        callback(undefined, './worker-source.mjs');
        return;
      }
      callback();
    },
  ],
  module: {
    parser: {
      javascript: {
        worker: true,
      },
    },
  },
});
