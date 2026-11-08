import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      hidePathInfo: false,
      minSize: 50,
      maxSize: 100,
    },
  },
});
