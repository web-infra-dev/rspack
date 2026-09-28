import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { SharePlugin } = sharing;

export default defineConfig({
  optimization: {},
  plugins: [
    new SharePlugin({
      shared: ['shared'],
    }),
  ],
});
