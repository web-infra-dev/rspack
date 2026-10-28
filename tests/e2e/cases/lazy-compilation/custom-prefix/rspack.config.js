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
    entries: true,
    imports: true,
    // Set custom prefix for lazy compilation
    prefix: '/custom-lazy-endpoint-',
  },
  devServer: {
    hot: true,
  },
};
