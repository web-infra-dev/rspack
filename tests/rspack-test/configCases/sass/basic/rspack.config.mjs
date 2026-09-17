/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/i,
        use: [{ loader: 'sass-loader' }],
        type: 'css',
      },
    ],
  },
};
