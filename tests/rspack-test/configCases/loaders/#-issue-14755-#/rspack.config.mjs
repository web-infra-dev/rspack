/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.my$/,
        loader: 'regexp-#-loader',
      },
    ],
  },
};
