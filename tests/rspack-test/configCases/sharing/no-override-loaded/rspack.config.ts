import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { SharePlugin } = sharing;

export default defineConfig({
  output: {
    uniqueName: 'b',
  },
  plugins: [
    new SharePlugin({
      shared: ['package'],
    }),
  ],
});
