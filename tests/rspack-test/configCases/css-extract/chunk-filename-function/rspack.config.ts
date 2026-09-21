import assert from 'node:assert/strict';
import { defineConfig } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  mode: 'development',
  target: 'web',
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
      chunkFilename: ({ chunk }) => {
        assert(chunk);
        return `expected.${chunk.name}.css`;
      },
    }),
  ],
});
