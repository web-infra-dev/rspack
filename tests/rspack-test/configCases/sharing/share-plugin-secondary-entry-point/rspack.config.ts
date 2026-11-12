import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { SharePlugin } = sharing;

export default defineConfig({
  mode: 'development',
  devtool: false,
  plugins: [
    new SharePlugin({
      shared: {
        '@scope/pkg': {},
        '@scope/pkg/styles': {},
      },
    }),
  ],
});
