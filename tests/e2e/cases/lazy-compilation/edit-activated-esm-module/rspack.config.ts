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
  cache: false,
  incremental: false,
  output: {
    module: true,
    chunkFormat: 'module',
    chunkLoading: 'import',
  },
  optimization: {
    splitChunks: false,
  },
  plugins: [new rspack.HtmlRspackPlugin({ scriptLoading: 'module' })],
  lazyCompilation: {
    entries: false,
    imports: true,
  },
  devServer: {
    hot: true,
  },
} satisfies Configuration;
