import type { LightningcssLoaderOptions } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

const lightningcss = (options: LightningcssLoaderOptions) => [
  './assert-loader-output.mjs',
  {
    loader: 'builtin:lightningcss-loader',
    options: {
      minify: false,
      ...options,
    } satisfies LightningcssLoaderOptions,
  },
];

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
        oneOf: [
          {
            test: /deep\.css$/,
            use: lightningcss({
              drafts: { customMedia: true },
              nonStandard: { deepSelectorCombinator: true },
              include: { customMediaQueries: true },
            }),
          },
          {
            resourceQuery: /^\?targets$/,
            use: lightningcss({
              drafts: { customMedia: true },
              targets: 'chrome >= 100',
            }),
          },
          {
            resourceQuery: /^\?no-draft$/,
            use: lightningcss({
              drafts: { customMedia: false },
              include: { customMediaQueries: true },
            }),
          },
          {
            resourceQuery: /^\?exclude$/,
            use: lightningcss({
              drafts: { customMedia: true },
              targets: 'chrome >= 100',
              exclude: { customMediaQueries: true },
            }),
          },
          {
            resourceQuery: /^\?no-targets$/,
            use: lightningcss({
              drafts: { customMedia: true },
            }),
          },
        ],
      },
    ],
  },
});
