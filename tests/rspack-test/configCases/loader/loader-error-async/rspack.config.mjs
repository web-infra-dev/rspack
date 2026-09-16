import path from 'node:path';

const file = path.resolve(import.meta.dirname, 'lib.js');

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: file,
        resourceQuery: /async/,
        use: [
          {
            loader: './async.js',
          },
        ],
      },
      {
        test: file,
        resourceQuery: /callback/,
        use: [
          {
            loader: './callback.js',
          },
        ],
      },
    ],
  },
};
