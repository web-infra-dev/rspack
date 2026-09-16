import path from 'node:path';

const resolve = (filename) => path.resolve(import.meta.dirname, filename);

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        exclude: /lib\.js/,
        use: [
          {
            loader: './loader.js',
          },
        ],
      },
      {
        exclude: resolve('index.js'),
        use: [
          {
            loader: './loader.js',
          },
        ],
      },
    ],
  },
};
