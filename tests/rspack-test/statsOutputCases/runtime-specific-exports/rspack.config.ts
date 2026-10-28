import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './example.js',
  optimization: {
    usedExports: true,
    providedExports: true,
  },
  stats: {
    assets: true,
    modules: true,
    usedExports: true,
    providedExports: true,
  },
});
