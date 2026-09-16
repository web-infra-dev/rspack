/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
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
