import { defineConfig } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

const config = (concatenateModules: boolean): Configuration => ({
  mode: 'production',
  optimization: {
    sideEffects: true,
    providedExports: true,
    usedExports: true,
    concatenateModules,
    minimize: false,
  },
});

export default defineConfig([config(true), config(false)]);
