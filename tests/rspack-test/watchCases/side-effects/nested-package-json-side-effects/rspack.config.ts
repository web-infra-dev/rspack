import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  cache: true,
  optimization: {
    minimize: false,
    concatenateModules: false,
    sideEffects: 'flag',
  },
  snapshot: { managedPaths: [] },
  experiments: { cache: { type: 'memory' } },
});
