import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  stats: {
    assets: true,
    modules: true,
    hash: false,
    moduleTrace: true,
    errorDetails: false,
  },
});
