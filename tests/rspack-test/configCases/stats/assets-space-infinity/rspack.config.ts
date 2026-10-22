import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  entry: './index.js',
  stats: {
    assetsSpace: Infinity,
  },
});
