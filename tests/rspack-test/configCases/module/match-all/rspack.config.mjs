/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        resourceQuery: /inline/,
        type: 'asset/inline',
      },
    ],
  },
};
