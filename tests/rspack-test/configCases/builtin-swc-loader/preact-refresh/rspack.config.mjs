import { rspack } from '@rspack/core';
import { PreactRefreshRspackPlugin } from '@rspack/plugin-preact-refresh';
import sources from 'webpack-sources';

const { ConcatSource, RawSource } = sources;
/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.jsx',
  mode: 'development',
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
  },
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.jsx$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
          jsc: {
            transform: {
              react: {
                runtime: 'classic',
                pragma: 'React.createElement',
                pragmaFrag: 'React.Fragment',
                throwIfNamespace: true,
                useBuiltins: false,
              },
            },
          },
        },
      },
    ],
  },
  plugins: [
    new rspack.HotModuleReplacementPlugin(),
    new PreactRefreshRspackPlugin(),
    new rspack.DefinePlugin({
      STUB: JSON.stringify('<div></div>'),
    }),
  ],
};
