import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  externals: {
    test: 'commonjs very-very-very-very-long-external-module-readable-identifier-it-should-be-truncated',
  },
  stats: {
    assets: true,
    modules: true,
  },
});
