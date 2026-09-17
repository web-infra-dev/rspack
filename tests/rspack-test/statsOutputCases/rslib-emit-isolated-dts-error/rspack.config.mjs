import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

/** @type {import('@rspack/core').Configuration} */
export default {
  entry: {
    index: './index',
    reference: './reference',
  },
  stats: 'errors-warnings',
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
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
};
