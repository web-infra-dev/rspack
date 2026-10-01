import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    mode: 'development',
    output: {
      library: {
        name: 'lib',
        type: 'global',
      },
    },
  },
  {
    mode: 'development',
    devtool: false,
    output: {
      library: {
        name: 'lib',
        type: 'global',
      },
    },
  },
]);
