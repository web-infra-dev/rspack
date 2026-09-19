import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      publicPath: '/assets/',
      favicon: path.resolve(import.meta.dirname, './static/favicon.ico'),
    }),
  ],
});
