import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    // TODO https://github.com/webpack/webpack/issues/16599
    chunkFilename: '[id].[hash].js',
  },
});
