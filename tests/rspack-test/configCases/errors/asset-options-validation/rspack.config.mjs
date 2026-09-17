/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.txt$/,
        type: 'asset/inline',
        generator: {
          filename: '[name].txt',
        },
      },
    ],
  },
};
