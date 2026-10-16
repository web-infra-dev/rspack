import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  cache: false,
  optimization: {
    minimize: false,
    concatenateModules: false,
    sideEffects: 'flag',
  },
  snapshot: { managedPaths: [] },
});
