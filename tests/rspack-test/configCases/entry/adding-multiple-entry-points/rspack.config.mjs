import { EntryPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: () => ({}),
  optimization: {
    runtimeChunk: true,
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].chunk.js',
  },
  target: 'web',
  plugins: [
    new EntryPlugin(import.meta.dirname, './fail', 'main'),
    new EntryPlugin(import.meta.dirname, './ok', 'main'),
    new EntryPlugin(import.meta.dirname, './fail', 'main'),
  ],
};
