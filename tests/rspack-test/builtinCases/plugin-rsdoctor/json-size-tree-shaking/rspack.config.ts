import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RsdoctorPlugin },
} = rspack;

export default defineConfig({
  builtins: {
    define: {
      'process.env.NODE_ENV': "'production'",
    },
  },
  optimization: {
    providedExports: true,
    usedExports: true,
    sideEffects: false,
  },
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: true, // Enable module sources feature to collect JSON sizes
      chunkGraphFeatures: true,
    }),
  ],
});
