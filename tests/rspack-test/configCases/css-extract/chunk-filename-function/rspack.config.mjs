import { CssExtractRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
  },
  mode: 'development',
  target: 'web',
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
      chunkFilename: ({ chunk }) => `expected.${chunk.name}.css`,
    }),
  ],
};
