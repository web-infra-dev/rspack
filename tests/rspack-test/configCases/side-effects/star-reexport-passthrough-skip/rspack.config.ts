import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    sideEffects: true,
    usedExports: true,
    providedExports: true,
    concatenateModules: false,
    minimize: false,
  },
});
