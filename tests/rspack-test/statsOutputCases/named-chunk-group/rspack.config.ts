import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  stats: {
    all: false,
    entrypoints: true,
    chunkGroups: true,
  },
});
