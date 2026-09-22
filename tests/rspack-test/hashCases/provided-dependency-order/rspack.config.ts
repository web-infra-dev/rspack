import { defineConfig } from '@rspack/cli';
import { type Configuration } from '@rspack/core';
import NodePolyfillPlugin from 'node-polyfill-webpack-plugin';
import path from 'node:path';

function config(subpath: string, realContentHash = false): Configuration {
  return {
    mode: 'development',
    devtool: false,
    context: import.meta.dirname,
    entry: './index.js',
    output: {
      path: path.resolve(import.meta.dirname, `./dist/${subpath}`),
      filename: '[name].[contenthash]-[contenthash:6].js',
    },
    optimization: {
      realContentHash,
    },
    plugins: [new NodePolyfillPlugin()],
  };
}

export default defineConfig([
  config('a'),
  config('b'),
  config('c'),
  config('d'),
  config('rch-a', true),
  config('rch-b', true),
  config('rch-c', true),
  config('rch-d', true),
]);
