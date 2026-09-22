import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  output: {
    filename: '[id].xxxx.js',
    chunkFilename: '[id].xxxx.js',
  },
  stats: {
    assets: true,
    modules: true,
  },
});
