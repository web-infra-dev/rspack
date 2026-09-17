import { rspack } from '@rspack/core';

/**
 * @type {import("@rspack/core").Configuration}
 */
export default {
  entry: {
    a: './a',
    b: './b',
    main: './index',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        exclude: [/b\.js/],
      }),
    ],
  },
};
