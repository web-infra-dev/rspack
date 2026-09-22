import { defineConfig } from '@rspack/cli';
import type { PathData } from '@rspack/core';

export default defineConfig(
  [
    {
      output: {
        publicPath: '/static/[hash:11]/',
      },
    },
    {
      output: {
        publicPath: '/static/[fullhash:11]/',
      },
    },
    {
      output: {
        publicPath: () => '/static/[hash:11]/',
      },
    },
    {
      output: {
        publicPath: () => '/static/[fullhash:11]/',
      },
    },
    {
      output: {
        publicPath: ({ hash }: PathData) => {
          return `/static/${hash!.slice(0, 11)}/`;
        },
      },
    },
  ].map((v) => ({ mode: 'development', ...v })),
);
