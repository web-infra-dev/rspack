/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.png$/,
        generator: {
          filename: '[name][ext]',
        },
        type: 'asset/resource',
      },
    ],
  },
};
