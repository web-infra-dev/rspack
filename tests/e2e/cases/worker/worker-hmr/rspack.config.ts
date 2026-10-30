import { defineConfig } from '@rspack/cli';
import rspack from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: './src/index.js',
  },
  devtool: false,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin({ template: './src/index.html' })],
  devServer: {
    hot: true,
  },
});
