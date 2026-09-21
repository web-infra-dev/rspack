import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import { PreactRefreshRspackPlugin } from '@rspack/plugin-preact-refresh';

export default defineConfig({
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
});
