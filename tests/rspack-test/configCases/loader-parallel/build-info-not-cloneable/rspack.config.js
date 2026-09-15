/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
  context: __dirname,
  module: {
    rules: [
      {
        test: /lib\.js/,
        use: [
          { loader: './worker-loader.js', parallel: true, options: {} },
          { loader: './seed-loader.js' },
        ],
      },
    ],
  },
};
