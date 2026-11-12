import { defineConfig } from '@rspack/cli';
import { type Configuration } from '@rspack/core';
import path from 'node:path';

function config(subpath: string, realContentHash = false): Configuration {
  return {
    entry: `./index.js`,
    context: path.resolve(import.meta.dirname, subpath),
    output: {
      path: path.resolve(import.meta.dirname, `dist/${subpath}`),
      filename: '[name].[contenthash].js',
      chunkFilename: '[name].js',
    },
    optimization: {
      realContentHash,
      moduleIds: 'named',
      minimize: false,
      runtimeChunk: {
        name: 'runtime',
      },
    },
  };
}

export default defineConfig([
  config('version0'),
  config('version0-copy'),
  config('version1'),
  config('rch-version0', true),
  config('rch-version0-copy', true),
  config('rch-version1', true),
]);
