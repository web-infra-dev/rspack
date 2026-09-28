import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    entry: './entry',
  },
  stats: {
    assets: true,
    modules: true,
  },
});
