import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    'node:fs': 'module node:path',
    'node:url': 'module-import node:url',
  },
  plugins: [
    new RslibPlugin({
      autoCjsNodeBuiltin: true,
    }),
  ],
};
