import type { LightningcssLoaderOptions } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
        generator: {
          exportsOnly: false,
        },
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
  output: { cssFilename: 'style.css' },
});
