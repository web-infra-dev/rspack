import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [true, false].map((arrowFunction, index) => ({
    target: 'node',
    mode: 'development',
    devtool: false,
    output: {
      module: true,
      iife: true,
      filename: `bundle${index}.mjs`,
      environment: { topLevelAwait: true, arrowFunction },
    },
  })),
);
