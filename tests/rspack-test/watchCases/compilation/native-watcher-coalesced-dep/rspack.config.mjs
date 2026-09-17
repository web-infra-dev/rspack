import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /late\.js$/,
        use: path.resolve(import.meta.dirname, 'coalesce.loader.js'),
      },
    ],
  },
};
