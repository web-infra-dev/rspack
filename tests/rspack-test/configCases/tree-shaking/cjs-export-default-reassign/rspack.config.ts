import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'node',
  optimization: {
    // innerGraph default-on is what triggered #14589; assert it explicitly.
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    providedExports: true,
    minimize: false,
  },
});
