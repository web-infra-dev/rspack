import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'node',
  optimization: {
    innerGraph: true,
    minimize: false,
    sideEffects: true,
    usedExports: true,
  },
  experiments: {
    pureFunctions: true,
  },
});
