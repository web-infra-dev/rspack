import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'async-node14',
  entry: './index.js',
  optimization: {
    chunkIds: 'named',
  },
  output: {
    filename: 'bundle.js',
  },
});
