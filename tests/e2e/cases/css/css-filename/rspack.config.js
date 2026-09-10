import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  mode: 'development',
  entry: './src/index.js',
  output: {
    cssFilename: 'css/[name].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'css/auto',
      },
    ],
  },
  devtool: false,
  devServer: {
    hot: true,
  },
  stats: 'none',
  infrastructureLogging: {
    debug: false,
  },
  plugins: [new rspack.HtmlRspackPlugin()],
  watchOptions: {
    poll: 1000,
  },
};
