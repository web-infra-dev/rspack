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
          { loader: './worker-loader.mjs', parallel: true, options: {} },
          { loader: './seed-loader.mjs' },
        ],
      },
    ],
  },
};
