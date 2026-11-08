import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  stats: {
    all: false,
    modules: true,
    nestedModules: true,
    orphanModules: true,
    optimizationBailout: true,
  },
});
