import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    chunkFilename: 'chunk-[name].js',
  },
  optimization: {
    chunkIds: 'named',
  },
});
