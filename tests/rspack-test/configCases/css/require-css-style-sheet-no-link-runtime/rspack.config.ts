import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  target: 'web',
  mode: 'development',
  devtool: false,
  plugins: [new rspack.HotModuleReplacementPlugin()],
  experiments: {
    css: true,
  },
  output: {
    // The require chunk-loading runtime cannot load stylesheet links. Even
    // though the style-sheet export type asks for the css runtime, the link
    // and loading handlers must stay out, matching webpack.
    chunkFormat: 'commonjs',
    chunkLoading: 'require',
    cssFilename: '[name].css',
    cssChunkFilename: '[name].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
      {
        test: /style\.css$/,
        type: 'css/auto',
        parser: {
          exportType: 'css-style-sheet',
        },
      },
    ],
  },
});
