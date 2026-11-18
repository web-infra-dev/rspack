import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    chunkFilename: 'chunk.[id].[contenthash].js',
    hashDigestLength: 1,
  },
  optimization: {
    realContentHash: false,
  },
});
