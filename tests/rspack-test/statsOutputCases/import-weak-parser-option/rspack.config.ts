import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    entry: './entry',
  },
  module: {
    parser: {
      javascript: {
        dynamicImportMode: 'weak',
      },
    },
  },
  stats: {
    assets: true,
    modules: true,
  },
});
