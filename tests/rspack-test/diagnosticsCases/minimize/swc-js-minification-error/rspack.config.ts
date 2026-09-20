import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';
export default defineConfig({
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
});
