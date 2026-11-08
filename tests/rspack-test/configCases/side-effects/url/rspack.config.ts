import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    sideEffects: true,
    innerGraph: true,
  },
});
