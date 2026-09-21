import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

function config(version: string) {
  return defineConfig({
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
  });
}

export default defineConfig([config('version0'), config('version1')]);
