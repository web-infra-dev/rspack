/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/inline',
      },
    ],
  },
};
