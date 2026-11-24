import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  devtool: false,
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.style\.css$/,
        type: 'css/auto',
        parser: {
          exportType: 'style',
        },
      },
      {
        test: /\.css$/,
        exclude: /\.style\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
