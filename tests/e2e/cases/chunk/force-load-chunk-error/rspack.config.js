import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  entry: './src/index.js',
  stats: 'none',
  mode: 'development',
  lazyCompilation: false,
  optimization: {
    splitChunks: {
      cacheGroups: {
        hooks: {
          test: /hooks\.js$/,
          name: 'hooks',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
  plugins: [new rspack.HtmlRspackPlugin()],
};
