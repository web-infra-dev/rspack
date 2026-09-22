import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  stats: {
    assets: true,
    modules: true,
    orphanModules: true,
    nestedModules: true,
    usedExports: true,
    reasons: true,
  },
});
