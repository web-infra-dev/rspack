import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: {
    a: { import: './index.js?a', filename: '[name].js' },
    b: { import: './index.js?b', filename: '[name].js' },
    c: { import: './index.js?c', filename: '[name].js' },
    d: { import: './index.js?d', filename: '[name].js' },
  },
  output: {
    filename: '[name].[contenthash].js',
  },
  plugins: [new rspack.HotModuleReplacementPlugin()],
});
