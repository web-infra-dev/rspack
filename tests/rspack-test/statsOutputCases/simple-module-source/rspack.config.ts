import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  mode: 'production',

  output: {
    filename: 'bundle.js',
  },
  stats: {
    assets: true,
    modules: true,
    builtAt: false,
    timings: false,
    source: true,
    version: false,
  },
});
