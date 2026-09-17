import { rspack } from '@rspack/core';

/**
 * @type {import("@rspack/core").Configuration}
 */
export default {
  entry: {
    a: './a',
    main: './index',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: true,
  },
  plugins: [
    new rspack.SwcJsMinimizerRspackPlugin({
      minimizerOptions: {
        minify: false,
        mangle: false,
      },
    }),
  ],
};
