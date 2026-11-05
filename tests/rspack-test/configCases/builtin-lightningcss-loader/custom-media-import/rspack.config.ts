import type { LightningcssLoaderOptions } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
        use: [
          './assert-loader-output.mjs',
          {
            loader: 'builtin:lightningcss-loader',
            options: {
              minify: false,
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
