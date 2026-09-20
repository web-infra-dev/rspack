import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [new rspack.optimize.LimitChunkCountPlugin({ maxChunks: 1 })],
});
