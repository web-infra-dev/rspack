/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
      },
      {
        test: /\.svg$/,
        resourceQuery: /inline/,
        type: 'asset/inline',
      },
      {
        test: /\.png$/,
        resourceQuery: /inline/,
        type: 'asset/inline',
      },
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
};
