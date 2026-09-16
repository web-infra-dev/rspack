import path from 'node:path';
import { rspack } from '@rspack/core';

function config(version) {
  return {
    mode: 'development',
    target: 'node',
    context: import.meta.dirname,
    entry: {
      runtime: {
        import: './runtime.js',
        filename: 'runtime.[chunkhash].js',
      },
      main: {
        import: `./${version}/index.js`,
        dependOn: 'runtime',
      },
    },
    output: {
      path: path.resolve(import.meta.dirname, `dist/${version}`),
      filename: '[name].[fullhash:base64:8].js',
      chunkFilename: '[id].js',
      chunkLoading: 'require',
      hotUpdateChunkFilename: '[id].hot-update.js',
      hotUpdateMainFilename: 'hot-update.json',
    },
    optimization: {
      chunkIds: 'named',
      moduleIds: 'named',
      minimize: false,
      realContentHash: false,
    },
    plugins: [new rspack.HotModuleReplacementPlugin()],
  };
}

/** @type {import('@rspack/core').Configuration[]} */
export default [config('version0'), config('version1')];
