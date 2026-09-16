import path from 'node:path';

const resolve = (filename) => path.resolve(import.meta.dirname, filename);

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: resolve('index.js'),
        use: [
          {
            loader: './test-loader.js',
          },
        ],
      },
    ],
  },
};
