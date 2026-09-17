import ImageMinimizerPlugin from 'image-minimizer-webpack-plugin';

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
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
          implementation: async (original) => original,
        },
        loader: false,
      }),
    ],
  },
};
