import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: [
      // Will trigger the issue.
      'data:text/javascript,import "core-js";',
      './src/index.css',
      './src/index.js',
    ],
  },
  stats: 'none',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [new rspack.HtmlRspackPlugin(), new rspack.CssExtractRspackPlugin()],
  experiments: {
    css: false,
  },
  lazyCompilation: true,
  devServer: {
    hot: true,
  },
});
