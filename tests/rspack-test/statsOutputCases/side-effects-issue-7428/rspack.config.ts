import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  entry: './main.js',
  optimization: {
    usedExports: true,
    sideEffects: true,
    concatenateModules: true,
  },
  stats: {
    assets: true,
    modules: true,
    orphanModules: true,
    nestedModules: true,
    usedExports: true,
    reasons: true,
  },
});
