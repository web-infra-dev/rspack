import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  output: {
    filename: 'main.js',
    chunkFilename: '[name].bundle.js',
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
