import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  mode: 'development',
  devtool: false,
  output: {
    cssChunkFilename: '[name].css',
  },
  node: {
    __dirname: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
