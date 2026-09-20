import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  devtool: 'source-map',
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
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
        type: 'css/auto',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
  plugins: [
    new rspack.DefinePlugin({
      CONTEXT: JSON.stringify(import.meta.dirname),
    }),
  ],
});
