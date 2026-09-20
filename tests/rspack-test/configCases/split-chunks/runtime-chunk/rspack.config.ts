import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  externals: {
    './dep-shared_js.js': 'commonjs ./dep-shared_js.js',
  },
  entry: {
    a: './a',
    b: './b',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        dep: {
          chunks: 'all',
          test: path.resolve(import.meta.dirname, 'shared.js'),
          enforce: true,
        },
      },
    },
  },
});
