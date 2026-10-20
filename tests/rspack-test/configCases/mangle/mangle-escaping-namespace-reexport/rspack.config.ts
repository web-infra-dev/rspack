import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].map((concatenateModules) => ({
    mode: 'production',
    optimization: {
      concatenateModules,
      sideEffects: false,
      minimize: false,
    },
  })),
);
