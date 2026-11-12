import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
      },
    ],
  },
});
