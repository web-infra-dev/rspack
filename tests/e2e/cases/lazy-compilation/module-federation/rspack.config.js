import { fileURLToPath } from 'node:url';
import { rspack } from '@rspack/core';
import { ReactRefreshRspackPlugin } from '@rspack/plugin-react-refresh';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  entry: './src/index.jsx',
  mode: 'development',
  devtool: false,
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.(jsx?|tsx?)$/,
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
              },
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new rspack.HtmlRspackPlugin({ template: './src/index.html' }),
    new rspack.container.ModuleFederationPlugin({
      name: 'host',
      remotes: {
        remote: 'remote@http://localhost:5679/remoteEntry.js',
      },
      // prevent init remote entry
      shareStrategy: 'loaded-first',
      shared: {
        react: {},
        'react-dom': {},
      },
      runtimePlugins: [
        fileURLToPath(import.meta.resolve('./runtime-plugin.js')),
      ],
    }),
    new ReactRefreshRspackPlugin(),
  ],
  lazyCompilation: true,
  devServer: {
    hot: true,
    port: 5678,
    devMiddleware: {
      writeToDisk: true,
    },
  },
};
