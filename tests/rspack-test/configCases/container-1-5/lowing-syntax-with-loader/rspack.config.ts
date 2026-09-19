import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

const { ModuleFederationPlugin } = rspack.container;

export default defineConfig({
  externals: {
    './container.js': 'commonjs ./container.js',
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                target: 'es2015',
              },
            },
          },
        ],
      },
    ],
  },
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        minimizerOptions: {
          format: {
            ecma: 6,
          },
        },
      }),
    ],
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      exposes: ['./module'],
    }),
  ],
});
