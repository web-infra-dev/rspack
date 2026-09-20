import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    chunkIds: 'deterministic',
  },
};
