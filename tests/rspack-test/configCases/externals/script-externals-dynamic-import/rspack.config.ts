import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    target: 'node',
    entry: {
      index: './index.js',
      case: './case.js',
    },
    output: {
      module: false,
      filename: '[name].js',
    },
    externals: {
      externals1:
        'script Externals1@https://unpkg.com/externals1@1.0.0/index.min.js',
    },
  },
]);
