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
          { loader: './worker-loader.js', parallel: true, options: {} },
          { loader: './seed-loader.js' },
        ],
      },
    ],
  },
};
