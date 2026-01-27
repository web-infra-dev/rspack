// @ts-check
import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: {
    main: './src/index.js',
  },
  target: ['browserslist:last 2 versions, > 0.2%, not dead, Firefox ESR'],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
      {
        test: /\.svg$/,
        type: 'asset',
      },
      {
        test: /\.js$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            /** @type {import('@rspack/core').SwcLoaderOptions} */
            options: {
              jsc: {
                parser: {
                  syntax: 'ecmascript',
                },
              },
            },
          },
        ],
      },
    ],
  },
  plugins: [new rspack.HtmlRspackPlugin({ template: './index.html' })],
});
