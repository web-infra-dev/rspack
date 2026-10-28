import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  stats: {
    all: false,
    assets: true,
    ids: true,
    entrypoints: true,
    chunkGroupChildren: true,
    chunkRelations: true,
    chunks: true,
  },
});
