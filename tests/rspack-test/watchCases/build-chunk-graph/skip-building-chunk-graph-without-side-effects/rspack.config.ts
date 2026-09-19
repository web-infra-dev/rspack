import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    sideEffects: true,
    usedExports: false,
    innerGraph: true,
  },
  incremental: {
    buildChunkGraph: true,
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        sideEffects: false,
      },
    ],
  },
});
