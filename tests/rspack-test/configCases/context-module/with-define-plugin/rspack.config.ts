import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.DefinePlugin({
      'process.env.DIR': JSON.stringify('sub'),
    }),
  ],
});
