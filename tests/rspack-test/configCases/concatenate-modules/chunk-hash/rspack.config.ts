import { LoaderOptionsPlugin } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

// Pass content through the loader context to keep the module identifier identical.
export default defineConfig([
  {
    mode: 'production',
    entry: './main.js',
    output: {
      chunkFilename: 'chunk.[chunkhash].js',
    },
    optimization: {
      concatenateModules: true,
      minimize: false,
    },
    plugins: [new LoaderOptionsPlugin({ mockContent: 'content1' })],
  },
  {
    mode: 'production',
    entry: './main.js',
    output: {
      chunkFilename: 'chunk.[chunkhash].js',
    },
    optimization: {
      concatenateModules: true,
      minimize: false,
    },
    plugins: [new LoaderOptionsPlugin({ mockContent: 'content2' })],
  },
]);
