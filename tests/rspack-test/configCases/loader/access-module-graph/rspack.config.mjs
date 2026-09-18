/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index.js/,
        use: [{ loader: './access-mg-loader.mjs' }],
      },
    ],
  },
};
