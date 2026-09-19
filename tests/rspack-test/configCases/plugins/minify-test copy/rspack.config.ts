import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
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
});
