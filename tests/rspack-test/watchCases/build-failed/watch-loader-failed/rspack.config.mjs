import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /.txt$/,
        loader: path.resolve(import.meta.dirname, './loader.js'),
      },
    ],
  },
  resolve: {
    extensions: ['...', '.txt'],
  },
};
