import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.mjs',
  resolve: {
    alias: {
      '@b': path.resolve(import.meta.dirname, './a'),
      xx: path.resolve(import.meta.dirname, './a'),
      ignored: path.resolve(import.meta.dirname, './a'),
    },
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        resolve: {
          alias: {
            ignored: false,
          },
        },
      },
    ],
  },
  optimization: {
    concatenateModules: false,
  },
};
