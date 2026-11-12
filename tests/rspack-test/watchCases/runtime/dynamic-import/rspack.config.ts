import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    chunkFilename: '[name].[chunkhash].js',
  },
});
