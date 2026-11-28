import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'production',
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        minimizerOptions: {
          compress: {
            keep_fnames: true,
            keep_classnames: true,
          },
          mangle: {
            keep_fnames: true,
            keep_classnames: true,
          },
        },
      }),
    ],
  },
});
