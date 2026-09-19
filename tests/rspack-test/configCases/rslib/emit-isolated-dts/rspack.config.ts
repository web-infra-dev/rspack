import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  externals: {
    'node:fs': 'node-commonjs node:fs',
    'node:path': 'node-commonjs node:path',
  },
  context: import.meta.dirname,
  entry: './index.ts',
  target: 'node',
  output: {
    library: {
      type: 'commonjs',
    },
  },
  module: {
    rules: [
      {
        test: /\.[cm]?ts$/,
        type: 'javascript/auto',
        use: {
          loader: 'builtin:swc-loader',
          options: {
            detectSyntax: 'auto',
            jsc: {
              experimental: {
                emitIsolatedDts: true,
              },
            },
          },
        },
      },
    ],
  },
  plugins: [
    new RslibPlugin({
      emitDts: {
        rootDir: import.meta.dirname,
        declarationDir: './dist/types',
      },
    }),
  ],
});
