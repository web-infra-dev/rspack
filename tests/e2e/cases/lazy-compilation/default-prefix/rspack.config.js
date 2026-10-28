import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  entry: {
    main: './src/index.js',
  },
  stats: 'none',
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  lazyCompilation: {
    entries: false,
    imports: true,
    // Using default prefix (not specifying prefix option)
  },
  devtool: false,
  devServer: {
    hot: true,
    devMiddleware: {
      writeToDisk: true,
    },
  },
};
