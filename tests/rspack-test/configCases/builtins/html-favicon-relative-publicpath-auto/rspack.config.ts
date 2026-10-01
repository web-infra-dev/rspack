import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    publicPath: 'auto',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      favicon: 'favicon.ico',
    }),
  ],
});
