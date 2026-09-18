/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    publicPath: '/public/',
  },
  entry: './index.js',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
      {
        test: /index\.js/,
        loader: './loader.mjs',
      },
    ],
  },
};
