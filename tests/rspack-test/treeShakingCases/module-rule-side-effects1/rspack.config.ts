import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  optimization: {
    sideEffects: true,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
  module: {
    rules: [
      {
        test: /b.js$/,
        sideEffects: false,
      },
    ],
  },
});
