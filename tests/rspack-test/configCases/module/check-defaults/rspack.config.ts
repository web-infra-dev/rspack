import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    output: {
      module: true,
    },
    devtool: false,
    target: 'web',
  },
  {
    output: {
      module: true,
    },
    devtool: false,
    target: 'node10',
  },
]);
