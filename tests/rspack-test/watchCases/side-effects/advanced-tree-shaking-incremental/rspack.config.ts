import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  cache: true,
  output: {
    pathinfo: true,
  },
  stats: {
    optimizationBailout: true,
    orphanModules: true,
  },
  optimization: {
    minimize: false,
    sideEffects: true,
    providedExports: true,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
    cache: {
      type: 'memory',
    },
  },
});
