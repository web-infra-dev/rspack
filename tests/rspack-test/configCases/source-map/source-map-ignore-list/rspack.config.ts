import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  devtool: false,
  plugins: [
    new rspack.SourceMapDevToolPlugin({
      filename: '[file].map',
      ignoreList: [/ignored\.js/],
    }),
  ],
});
