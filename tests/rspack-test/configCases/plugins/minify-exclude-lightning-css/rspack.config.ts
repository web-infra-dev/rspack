import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: {
    a: './a.js',
    b: './b.js',
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
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
    minimizer: [
      new rspack.LightningCssMinimizerRspackPlugin({
        exclude: [/b\.css/],
      }),
    ],
  },
});
