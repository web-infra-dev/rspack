const ImageMinimizerPlugin = require('image-minimizer-webpack-plugin');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    minimize: true,
    minimizer: [
      new ImageMinimizerPlugin({
        minimizer: {
          implementation: ImageMinimizerPlugin.sharpMinify,
          options: {
            encodeOptions: {
              png: { quality: 50 },
            },
          },
        },
        loader: false,
      }),
    ],
  },
};
