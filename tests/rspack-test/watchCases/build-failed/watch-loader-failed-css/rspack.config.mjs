import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /.txt$/,
        type: 'javascript/auto',
        use: [
          {
            loader: rspack.CssExtractRspackPlugin.loader,
          },
          {
            loader: 'css-loader',
            options: {
              modules: {
                namedExport: true,
                exportLocalsConvention: 'camel-case-only',
              },
            },
          },
          {
            loader: path.resolve(import.meta.dirname, './loader.mjs'),
          },
        ],
      },
    ],
  },
  plugins: [new rspack.CssExtractRspackPlugin()],
  experiments: {
    css: false,
  },
  resolve: {
    extensions: ['...', '.txt'],
  },
};
