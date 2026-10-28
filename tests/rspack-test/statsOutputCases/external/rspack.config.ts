import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  externals: {
    test: 'commonjs test',
  },
  stats: {
    assets: true,
    modules: true,
  },
});
