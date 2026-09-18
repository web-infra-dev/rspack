import path from 'node:path';

const context = `\\\\?\\${path.resolve(import.meta.dirname)}`;

/** @type {import('@rspack/core').RspackOptions} */
export default {
  context,
  entry: './index.js',
  resolve: {
    symlinks: false,
  },
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: [`${path.join(context, 'loader.mjs')}?loader-query`],
      },
    ],
  },
};
