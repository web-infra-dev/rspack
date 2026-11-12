import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  optimization: {
    inlineExports: true,
  },
  plugins: [
    new rspack.DefinePlugin({
      'typeof window': JSON.stringify('object'),
    }),
  ],
});
