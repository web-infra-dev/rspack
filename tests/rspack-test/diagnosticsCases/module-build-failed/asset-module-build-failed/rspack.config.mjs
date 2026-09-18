/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        use: [
          {
            loader: './my-loader.mjs',
          },
        ],
        type: 'asset',
      },
    ],
  },
};
