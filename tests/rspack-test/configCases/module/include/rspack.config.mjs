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
        include: (value) => value === resolve('lib.js'),
        use: './loader-3.js',
      },
      {
        include: /lib\.js/,
        use: [
          {
            loader: './loader-2.js',
          },
        ],
      },
      {
        include: resolve('lib.js'),
        use: [
          {
            loader: './loader-1.js',
          },
        ],
      },
    ],
  },
};
