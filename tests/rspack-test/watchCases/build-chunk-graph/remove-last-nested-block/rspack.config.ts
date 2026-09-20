import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
});
