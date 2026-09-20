import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: './index.js',
  target: 'web',
  optimization: {
    minimize: true,
    minimizer: [new rspack.LightningCssMinimizerRspackPlugin()],
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
