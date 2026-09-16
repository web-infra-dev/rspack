import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  mode: 'development',
  entry: './src/index.js',
  devServer: {
    hot: true,
  },
  stats: 'none',
  plugins: [new rspack.HtmlRspackPlugin()],
  watchOptions: {
    poll: 1000,
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
  optimization: {
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        a: {
          test: /a.css/,
          name: 'a',
        },
        b: {
          test: /b.css/,
          name: 'b',
        },
      },
    },
  },
};
