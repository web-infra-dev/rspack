import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  resolveLoader: {
    alias: {
      'aliased-transform-loader': `${path.join(import.meta.dirname, 'transform-loader.mjs')}?value=from-alias`,
    },
  },
  module: {
    rules: [
      {
        test: /\.never-match$/,
        use: [
          {
            loader: './transform-loader.mjs',
            ident: 'string-filter',
            options: { value: 'string-hit' },
            codeFilter: 'MARKER_STR',
            parallel: true,
          },
          {
            loader: './transform-loader.mjs',
            ident: 'regexp-filter',
            options: { value: 'regexp-hit' },
            codeFilter: /MARKER_REGEXP/,
            parallel: true,
          },
          {
            loader: 'aliased-transform-loader',
            ident: 'alias-meta-only',
            codeFilter: 'MARKER_ALIAS',
          },
        ],
      },
      {
        test: /\.alias-filter\.js$/,
        use: [
          {
            loader: 'aliased-transform-loader',
            codeFilter: 'MARKER_ALIAS',
          },
        ],
      },
      {
        test: /\.alias-cache\.js$/,
        use: [
          {
            loader: 'aliased-transform-loader',
            cache: true,
          },
        ],
      },
      {
        test: /\.alias-override\.js$/,
        use: [
          {
            loader: 'aliased-transform-loader',
            options: { value: 'from-options' },
            codeFilter: 'MARKER_ALIAS',
          },
        ],
      },
      {
        test: /\.virtual\.js$/,
        use: [
          './pitch-import-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'pitched-hit' },
            codeFilter: 'MARKER_PITCH',
            parallel: true,
          },
        ],
      },
    ],
  },
});
