import assert from 'node:assert/strict';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  output: {
    filename: ({ chunk }) => {
      assert(chunk);
      return chunk.name === 'main' ? 'js/main.js' : 'runtime.bundle.js';
    },
    chunkFilename: 'worker.bundle.js',
    library: {
      type: 'modern-module',
    },
  },
  module: {
    parser: {
      javascript: {
        worker: {
          url: 'new-url-relative',
        },
      },
    },
  },
});
