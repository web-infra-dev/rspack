import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    entry: './a',
    target: 'web',
    output: {
      filename: 'a.js',
      scriptType: 'module',
      publicPath: 'auto',
    },
    module: {
      rules: [
        {
          test: /\.png$/,
          type: 'asset/resource',
        },
      ],
    },
  },
  {
    entry: './index',
  },
]);
