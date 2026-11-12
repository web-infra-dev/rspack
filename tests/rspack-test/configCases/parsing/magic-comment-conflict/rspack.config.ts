import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  output: {
    chunkFilename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
  },
});
