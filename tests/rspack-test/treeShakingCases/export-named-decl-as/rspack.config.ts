import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    main: {
      import: ['./src/index.js'],
    },
  },
  optimization: {
    sideEffects: true,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
});
