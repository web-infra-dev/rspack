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
        test: /lib\.js/,
        use: [
          {
            loader: './loader-2.js',
          },
        ],
      },
      {
        test: resolve('lib.js'),
        use: [
          {
            loader: './loader-1.js',
          },
        ],
      },
      {
        test: /\.module\.less$/,
        type: 'css/module',
      },
      {
        test: /(?<!module).less$/,
        type: 'css',
      },
      {
        test: /\.svg$/i,
        type: 'asset',
      },
    ],
  },
};
