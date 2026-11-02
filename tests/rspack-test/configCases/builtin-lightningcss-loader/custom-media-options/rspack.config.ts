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
    // Keep the builtin CSS parser out of this case so the assertions observe
    // only the loader's custom-media behavior.
    parser: {
      css: {
        customMedia: false,
      },
      'css/auto': {
        customMedia: false,
      },
      'css/global': {
        customMedia: false,
      },
      'css/module': {
        customMedia: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        oneOf: [
          {
            test: /deep\.css$/,
            type: 'css',
            use: lightningcss({
              drafts: { customMedia: true },
              nonStandard: { deepSelectorCombinator: true },
              include: { customMediaQueries: true },
            }),
          },
          {
            resourceQuery: /^\?targets-css$/,
            type: 'css',
            use: lightningcss({
              drafts: { customMedia: true },
              targets: 'chrome >= 100',
            }),
          },
          {
            resourceQuery: /^\?targets-auto$/,
            type: 'css/auto',
            use: lightningcss({
              drafts: { customMedia: true },
              targets: 'chrome >= 100',
            }),
          },
          {
            resourceQuery: /^\?targets-global$/,
            type: 'css/global',
            use: lightningcss({
              drafts: { customMedia: true },
              targets: 'chrome >= 100',
            }),
          },
          {
            resourceQuery: /^\?targets-module$/,
            type: 'css/module',
            use: lightningcss({
              drafts: { customMedia: true },
              targets: 'chrome >= 100',
            }),
          },
          {
            resourceQuery: /^\?no-draft$/,
            type: 'css',
            use: lightningcss({
              drafts: { customMedia: false },
              include: { customMediaQueries: true },
            }),
          },
          {
            resourceQuery: /^\?exclude$/,
            type: 'css',
            use: lightningcss({
              drafts: { customMedia: true },
              targets: 'chrome >= 100',
              exclude: { customMediaQueries: true },
            }),
          },
          {
            resourceQuery: /^\?no-targets$/,
            type: 'css',
            use: lightningcss({
              drafts: { customMedia: true },
            }),
          },
        ],
      },
    ],
  },
});
