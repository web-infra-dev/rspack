import type { Configuration } from '@rspack/core';
import rspack from '@rspack/core';

export default {
  mode: 'development',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.mjs',
  },
  devtool: false,
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      scriptLoading: 'module',
    }),
  ],
  optimization: {
    minimize: false,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        split: {
          test: /split/,
          name: 'split',
          priority: 10,
          enforce: true,
        },
      },
    },
  },
} satisfies Configuration;
