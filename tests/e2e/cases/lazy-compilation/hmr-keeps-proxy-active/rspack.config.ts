import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: {
    main: './src/index.js',
  },
  mode: 'development',
  stats: 'none',
  devtool: false,
  plugins: [new rspack.HtmlRspackPlugin()],
  lazyCompilation: {
    entries: false,
    imports: true,
  },
  devServer: {
    hot: true,
  },
} satisfies Configuration;
