import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  stats: {
    all: false,
    assets: true,
    entrypoints: true,
    chunkGroupChildren: true,
    chunks: true,
  },
});
