/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  output: {
    publicPath: '/',
    filename: 'main.js',
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/auto',
        generator: {
          filename: 'custom/lib.js',
        },
      },
    ],
  },
};
