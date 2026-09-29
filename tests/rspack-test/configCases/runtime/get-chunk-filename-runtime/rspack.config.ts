import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: './a.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[runtime].[contenthash].[name].js',
  },
});
