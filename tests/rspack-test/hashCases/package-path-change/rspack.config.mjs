import path from 'node:path';

function config(subpath, realContentHash = false) {
  return {
    entry: `./index.js`,
    context: path.resolve(import.meta.dirname, subpath),
    output: {
      path: path.resolve(import.meta.dirname, `dist/${subpath}`),
      filename: '[name].[contenthash].js',
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

/** @type {import("@rspack/core").Configuration} */
export default [
  config('version0'),
  config('version0-copy'),
  config('version1'),
  config('rch-version0', true),
  config('rch-version0-copy', true),
  config('rch-version1', true),
];
