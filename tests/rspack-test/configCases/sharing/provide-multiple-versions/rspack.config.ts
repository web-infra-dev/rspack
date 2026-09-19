import { defineConfig } from '@rspack/cli';

import { sharing } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { ProvideSharedPlugin } = sharing;

export default defineConfig({
  plugins: [
    new ProvideSharedPlugin({
      provides: ['shared'],
    }),
  ],
});
