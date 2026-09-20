import { defineConfig } from '@rspack/cli';

import { HtmlRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    chunk1: {
      import: ['./chunk1.js'],
    },
    chunk2: {
      import: ['./chunk2.js'],
    },
    chunk3: {
      import: ['./chunk3.js'],
    },
  },
  plugins: [
    new HtmlRspackPlugin({
      template: 'index.html',
      chunks: ['chunk1', 'chunk2'],
      excludeChunks: ['chunk2'],
    }),
  ],
});
