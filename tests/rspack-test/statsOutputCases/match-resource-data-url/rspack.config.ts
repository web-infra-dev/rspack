import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  mode: 'development',
  stats: {
    all: false,
    modules: true,
  },
});
