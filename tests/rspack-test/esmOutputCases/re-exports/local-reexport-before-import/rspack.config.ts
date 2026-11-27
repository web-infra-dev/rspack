import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  optimization: {
    minimize: false,
    moduleIds: 'named',
    chunkIds: 'named',
    runtimeChunk: false,
  },
});
