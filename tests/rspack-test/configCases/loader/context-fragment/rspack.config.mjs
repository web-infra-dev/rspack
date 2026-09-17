import path from 'node:path';

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'a.js'),
        use: [
          {
            loader: './my-loader.js',
          },
        ],
      },
      {
        test: path.join(import.meta.dirname, 'b.js'),
        use: [
          {
            loader: './my-loader.js',
          },
        ],
      },
    ],
  },
};
