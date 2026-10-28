import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  mode: 'development',
  entry: './src/index.js',
  devServer: {
    hot: true,
  },
  stats: 'none',
  infrastructureLogging: {
    debug: false,
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
    }),
  ],
  watchOptions: {
    poll: 1000,
  },
};
