import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: './index.js',
  optimization: {
    splitChunks: false,
  },
  stats: {
    all: false,
    chunks: true,
    chunkModules: true,
    dependentModules: false,
  },
});
