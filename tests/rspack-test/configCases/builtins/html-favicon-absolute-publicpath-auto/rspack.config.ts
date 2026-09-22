import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    publicPath: 'auto',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      favicon: path.resolve(import.meta.dirname, 'favicon.ico'),
    }),
  ],
});
