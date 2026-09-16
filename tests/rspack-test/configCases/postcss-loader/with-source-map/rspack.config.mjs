import { rspack } from '@rspack/core';
import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
