/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset',
      },
    ],
  },
};
