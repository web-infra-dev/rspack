import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  target: 'node',
  module: { parser: { javascript: { requireAlias: true } } },
  plugins: [
    new rspack.DefinePlugin({
      'DEFINED.branch.value': JSON.stringify('defined'),
      THIS_VALUE: 'this.value',
    }),
  ],
});
