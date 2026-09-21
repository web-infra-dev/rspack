import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  stats: {
    assets: true,
    modules: true,
  },
  output: {
    filename: 'bundle.js',
  },
});
