import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    mangleExports: true,
    usedExports: true,
    providedExports: true,
    sideEffects: false, // disable reexports optimization
  },
});
