import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  entry: './src/index.js',
  mode: 'development',
  stats: 'none',
  devtool: false,
  optimization: {
    usedExports: false,
    sideEffects: 'flag',
  },
  plugins: [new rspack.HtmlRspackPlugin()],
  lazyCompilation: true,
  devServer: { hot: true },
});
