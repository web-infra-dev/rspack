import { defineConfig } from '@rspack/cli';

export default defineConfig({
  // Rspack does not support reordering compilation.modules; they are already unsorted.
  optimization: {
    sideEffects: true,
  },
});
