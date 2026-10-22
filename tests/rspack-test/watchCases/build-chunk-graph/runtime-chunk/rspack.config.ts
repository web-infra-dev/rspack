import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: true,
  },
  incremental: {
    buildChunkGraph: true,
  },
});
