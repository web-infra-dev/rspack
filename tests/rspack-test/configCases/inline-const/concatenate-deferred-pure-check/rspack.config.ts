import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    concatenateModules: true,
    inlineExports: true,
    minimize: false,
    providedExports: true,
    sideEffects: true,
    usedExports: true,
  },
});
