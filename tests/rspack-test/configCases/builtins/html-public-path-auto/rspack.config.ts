import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  target: 'web',
  externals: {
    path: "require('path')",
    fs: "require('fs')",
  },
  node: {
    __dirname: false,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      filename: 'main_page/index.html',
    }),
  ],
});
