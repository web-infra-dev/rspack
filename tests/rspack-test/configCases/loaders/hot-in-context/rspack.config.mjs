import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    // no hmr
  },
  {
    // with hmr
    plugins: [new rspack.HotModuleReplacementPlugin()],
  },
];
