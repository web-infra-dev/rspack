import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
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
});
