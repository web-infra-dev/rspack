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
        resource: /lib\.js/,
        use: [
          {
            loader: './loader-2.js',
          },
        ],
      },
      {
        resource: resolve('lib.js'),
        use: [
          {
            loader: './loader-1.js',
          },
        ],
      },
    ],
  },
};
