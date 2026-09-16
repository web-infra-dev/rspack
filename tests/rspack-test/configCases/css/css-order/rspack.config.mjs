import { CssExtractRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  devtool: false,
  target: 'web',
  entry: './index.js',
  mode: 'development',
  optimization: {
    concatenateModules: false,
  },
  experiments: {
    css: false,
  },
  module: {
    rules: [
      {
        test: /\.module\.css$/,
        type: 'javascript/auto',
        use: [
          {
            loader: CssExtractRspackPlugin.loader,
          },
          {
            loader: 'css-loader',
            options: {
              esModule: true,
              modules: {
                namedExport: false,
                localIdentName: '[name]',
              },
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
    }),
  ],
  node: {
    __dirname: false,
    __filename: false,
  },
};
