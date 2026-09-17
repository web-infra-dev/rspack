/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.txt$/,
        use: {
          loader: 'file-loader',
          options: {
            name: 'same-name.txt',
          },
        },
      },
    ],
  },
};
