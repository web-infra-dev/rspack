import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  optimization: {
    innerGraph: true,
    sideEffects: true,
    usedExports: true,
    providedExports: true,
  },
});
