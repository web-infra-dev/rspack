import { defineConfig } from '@rspack/cli';
import rspack from '@rspack/core';

export default defineConfig({
  entry: './src/index.js',
  context: import.meta.dirname,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  optimization: {
    emitOnErrors: false,
  },
  devServer: {
    hot: true,
  },
});
