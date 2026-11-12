import path from 'node:path';
import { defineConfig } from '@rspack/cli';

function config(subpath: string, filename: string) {
  return defineConfig({
    entry: {
      main: {
        import: './index.js',
        filename: '[name].[chunkhash].js',
      },
    },
    output: {
      path: path.resolve(import.meta.dirname, `dist/${subpath}`),
      filename,
    },
    target: 'node',
    optimization: {
      realContentHash: false,
      moduleIds: 'named',
      chunkIds: 'named',
      minimize: false,
    },
  });
}

export default defineConfig([
  config('a', '[name].[chunkhash].js'),
  config('b', '[name].[contenthash].js'),
]);
