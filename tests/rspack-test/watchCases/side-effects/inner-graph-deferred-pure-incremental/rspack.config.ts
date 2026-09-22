import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  cache: true,
  output: {
    pathinfo: true,
  },
  stats: {
    orphanModules: true,
  },
  optimization: {
    minimize: false,
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
    cache: {
      type: 'memory',
    },
  },
});
