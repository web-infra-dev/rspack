import path from 'node:path';
import { CssExtractRspackPlugin } from '@rspack/core';

/**@type {import('@rspack/core').Configuration} */
export default {
  entry: './src/index.js',
  experiments: {
    css: false,
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
    }),
  ],
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(import.meta.dirname, 'src'),
    },
  },
};
