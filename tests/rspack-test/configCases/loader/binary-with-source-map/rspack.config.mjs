import path from 'node:path';

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'logo.png'),
        use: ['./empty-loader.mjs'],
        type: 'asset/resource',
      },
    ],
  },
};
