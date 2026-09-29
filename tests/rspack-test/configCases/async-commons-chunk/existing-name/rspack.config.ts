import { defineConfig } from '@rspack/cli';

export default defineConfig({
  performance: {
    hints: false,
  },
  optimization: {
    splitChunks: {
      minSize: 1,
    },
    chunkIds: 'named',
  },
});
