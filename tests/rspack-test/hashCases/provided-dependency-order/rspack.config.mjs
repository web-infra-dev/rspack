import NodePolyfillPlugin from 'node-polyfill-webpack-plugin';
import path from 'node:path';

function config(subpath, realContentHash = false) {
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

/** @type {import("@rspack/core").Configuration} */
export default [
  config('a'),
  config('b'),
  config('c'),
  config('d'),
  config('rch-a', true),
  config('rch-b', true),
  config('rch-c', true),
  config('rch-d', true),
];
