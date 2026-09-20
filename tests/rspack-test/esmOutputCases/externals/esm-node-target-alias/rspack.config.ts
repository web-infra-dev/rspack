import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  externals: {
    'node:fs': 'module node:path',
    'node:url': 'module-import node:url',
  },
  plugins: [
    new RslibPlugin({
      autoCjsNodeBuiltin: true,
    }),
  ],
});
