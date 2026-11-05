import type { LightningcssLoaderOptions } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
        use: [
          {
            loader: 'builtin:lightningcss-loader',
            options: {
              drafts: {
                customMedia: true,
              },
              include: {
                customMediaQueries: true,
              },
            } satisfies LightningcssLoaderOptions,
          },
        ],
      },
    ],
  },
});
