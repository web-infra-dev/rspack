import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'node',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    providedExports: true,
    minimize: false,
  },
});
