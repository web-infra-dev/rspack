import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      publicPath: '/',
      favicon: './资源/favicon-图标.ico',
    }),
  ],
});
