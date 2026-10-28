import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  output: {
    chunkFilename: '[name].js',
  },
  stats: {
    modules: true,
    timings: false,
    hash: false,
    entrypoints: false,
    assets: false,
    errorDetails: false,
    moduleTrace: true,
  },
});
