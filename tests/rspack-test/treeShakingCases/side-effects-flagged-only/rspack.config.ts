import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    main: {
      import: ['./index.js'],
    },
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
  optimization: {
    sideEffects: 'flag',
  },
});
