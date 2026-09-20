import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  externals: {
    'node:fs': 'node-commonjs node:fs',
    'node:path': 'node-commonjs node:path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          rspack.CssExtractRspackPlugin.loader,
          'css-loader',
          {
            loader: 'builtin:lightningcss-loader',
            /** @type {import("@rspack/core").LightningcssLoaderOptions} */
            options: {
              targets: ['Edge >= 12'],
            },
          },
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: [
                  fileURLToPath(import.meta.resolve('postcss-pxtorem')),
                ],
              },
            },
          },
        ],
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: 'bundle0.css',
    }),
  ],
});
