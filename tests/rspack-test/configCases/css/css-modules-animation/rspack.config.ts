import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    generator: {
      'css/auto': {
        localIdentName: '[path][name]-[local]',
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  optimization: {
    minimize: true,
    minimizer: [new rspack.LightningCssMinimizerRspackPlugin()],
    providedExports: true,
    usedExports: true,
  },
});
