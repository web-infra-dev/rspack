import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    target: 'node',
    entry: {
      index: './index.js',
      case: './case.js',
    },
    externalsType: 'commonjs-import',
    output: {
      module: false,
      filename: '[name].js',
    },
    externals: {
      external1: 'external1-alias',
      external2: 'external2-alias',
    },
  },
]);
