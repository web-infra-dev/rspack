import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      publicPath: '/assets/',
      favicon: './static/favicon.ico',
    }),
  ],
});
