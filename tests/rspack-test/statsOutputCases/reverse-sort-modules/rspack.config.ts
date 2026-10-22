import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  performance: false,
  stats: {
    assets: true,
    modules: true,
    modulesSpace: Infinity,
    modulesSort: '!name',
  },
});
