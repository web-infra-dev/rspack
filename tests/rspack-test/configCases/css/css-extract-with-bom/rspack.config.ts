import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  entry: {
    main: './index.js',
    raw: './raw.js',
  },
  output: {
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /a\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
      {
        // `b.css` is prefixed with a BOM by a loader, so the BOM ends up in the
        // middle of the concatenated bundle instead of at offset 0.
        test: /b\.css$/,
        type: 'javascript/auto',
        use: [
          rspack.CssExtractRspackPlugin.loader,
          'css-loader',
          fileURLToPath(import.meta.resolve('./bom-loader.mjs')),
        ],
      },
    ],
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: '[name].css',
    }),
  ],
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.LightningCssMinimizerRspackPlugin({
        exclude: [/raw\.css/],
      }),
    ],
  },
  experiments: {
    css: false,
  },
});
