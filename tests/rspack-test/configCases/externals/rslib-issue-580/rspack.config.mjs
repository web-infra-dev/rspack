import { rspack } from '@rspack/core';

/** @type {function(any, any): import("@rspack/core").Configuration[]} */
export default (env, { testPath }) => {
  return {
    externals: [/.*foo.*/],
    externalsType: 'module',
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
      new rspack.CopyRspackPlugin({
        patterns: ['./a/**/*', './_a/**/*'],
      }),
    ],
  };
};
