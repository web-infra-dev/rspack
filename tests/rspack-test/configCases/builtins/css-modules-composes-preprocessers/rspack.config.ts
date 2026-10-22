import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
      },
      {
        test: /\.scss$/,
        use: [{ loader: 'sass-loader' }],
        type: 'css/module',
      },
      {
        test: /\.less$/,
        use: [{ loader: 'less-loader' }],
        type: 'css/module',
      },
    ],
  },
});
