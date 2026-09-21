import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';

export default defineConfig((_env, { testPath }) => {
  return {
    entry: {
      consume: './consume.js',
      main: './index.js',
    },
    resolve: {
      alias: {
        library: path.resolve(
          testPath,
          '../0-concatenation-tree-shaking/main.mjs',
        ),
      },
    },
    output: {
      module: true,
      chunkFormat: 'module',
      filename: '[name].mjs',
    },
    optimization: {
      minimize: true,
      concatenateModules: true,
    },
    plugins: [
      new rspack.SwcJsMinimizerRspackPlugin({
        minimizerOptions: {
          minify: false,
          mangle: false,
        },
      }),
    ],
  };
});
