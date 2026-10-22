import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    externals: {
      path: 'node-commonjs path',
    },
    target: 'web',
    mode: 'development',

    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
  {
    externals: {
      path: 'node-commonjs path',
    },
    target: 'web',
    mode: 'production',

    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  },
]);
