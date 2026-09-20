import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    entry1: './entry-1.js',
    entry2: './entry-2.js',
    entry3: './entry-3.js',
  },
  stats: {
    assets: true,
    modules: true,
  },
});
