import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      publicPath: '/',
      favicon: 'favicon.ico',
    }),
  ],
});
