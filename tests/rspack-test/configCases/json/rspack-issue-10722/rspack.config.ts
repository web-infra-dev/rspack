import { defineConfig } from '@rspack/cli';

import { CopyRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  plugins: [
    new CopyRspackPlugin({
      patterns: [{ from: 'test.mjs' }],
    }),
  ],
});
