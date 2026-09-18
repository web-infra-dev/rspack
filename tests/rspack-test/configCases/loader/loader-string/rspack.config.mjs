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
            loader: './my-loader.mjs',
          },
        ],
      },
    ],
  },
};
