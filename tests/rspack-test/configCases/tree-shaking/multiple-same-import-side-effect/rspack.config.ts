import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,

  optimization: {
    sideEffects: true,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify('production'),
    }),
  ],
});
