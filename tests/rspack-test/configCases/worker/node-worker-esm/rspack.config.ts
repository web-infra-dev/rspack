import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: './index.js',
  optimization: {
    chunkIds: 'named',
  },
  output: {
    module: true,
    filename: 'bundle.mjs',
  },
});
