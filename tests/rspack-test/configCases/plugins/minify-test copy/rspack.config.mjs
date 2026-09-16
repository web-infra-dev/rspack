import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    a: './a',
    a2: './a2',
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
        test: [/a\d?\.js/],
        exclude: [/a\.js/],
      }),
    ],
  },
};
