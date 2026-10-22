import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  mode: 'production',
  module: {
    parser: {
      javascript: {
        exportsPresence: 'auto',
      },
    },
  },
  stats: {
    assets: true,
    modules: true,
    warningsSpace: 0,
    warnings: true,
  },
});
