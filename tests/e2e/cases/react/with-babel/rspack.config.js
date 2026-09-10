import { fileURLToPath } from 'node:url';
import { rspack } from '@rspack/core';
import { ReactRefreshRspackPlugin } from '@rspack/plugin-react-refresh';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  mode: 'development',
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
      {
        test: /\.jsx$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              jsc: {
                transform: {
                  react: {
                    runtime: 'automatic',
                    development: true,
                    refresh: true,
                  },
                },
                externalHelpers: true,
              },
            },
          },
          {
            loader: 'babel-loader',
            options: {
              presets: [['@babel/preset-react', { runtime: 'automatic' }]],
              plugins: [
                fileURLToPath(import.meta.resolve('react-refresh/babel')),
              ],
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    new ReactRefreshRspackPlugin(),
  ],
  entry: './src/index.jsx',
  devServer: {
    hot: true,
  },
  stats: 'none',
  infrastructureLogging: {
    debug: false,
  },
  watchOptions: {
    poll: 1000,
  },
};
