import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.scss$/,
        use: [{ loader: 'sass-loader' }],
        type: 'css/module',
      },
    ],
  },
});
