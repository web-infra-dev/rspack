import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [new rspack.optimize.LimitChunkCountPlugin({ maxChunks: 1 })],
};
