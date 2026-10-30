import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

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
    // serverUrl will be set dynamically in the test to point to a different port
  },
  devtool: false,
  devServer: {
    hot: true,
    devMiddleware: {
      writeToDisk: true,
    },
  },
} satisfies Configuration;
