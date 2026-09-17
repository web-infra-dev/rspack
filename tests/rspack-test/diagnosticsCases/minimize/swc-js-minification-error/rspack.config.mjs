import { rspack } from '@rspack/core';
export default {
  entry: './index.js',
  target: 'web',
  devtool: false,
  optimization: {
    minimize: true,
    minimizer: [new rspack.SwcJsMinimizerRspackPlugin()],
  },
  module: {
    noParse: /index\.js$/,
  },
};
