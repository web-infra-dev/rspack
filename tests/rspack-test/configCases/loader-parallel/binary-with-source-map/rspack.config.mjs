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
        use: [{ loader: './empty-loader.js', parallel: true, options: {} }],
        type: 'asset/resource',
      },
    ],
  },
};
