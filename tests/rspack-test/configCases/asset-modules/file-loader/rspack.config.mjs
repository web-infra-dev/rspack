/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.png$/,
        loader: 'file-loader',
        options: {
          name: 'file-loader.[ext]',
        },
      },
    ],
  },
};
