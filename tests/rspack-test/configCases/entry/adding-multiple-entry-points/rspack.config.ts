import { defineConfig } from '@rspack/cli';
import { EntryPlugin } from '@rspack/core';

export default defineConfig({
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
});
