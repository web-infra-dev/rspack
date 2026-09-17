import { rspack } from '@rspack/core';
import path from 'node:path';

/** @type {function(any, any): import("@rspack/core").Configuration[]} */
export default (env, { testPath }) => {
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
};
