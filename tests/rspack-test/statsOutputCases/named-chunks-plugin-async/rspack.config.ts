import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: { chunkIds: 'named' },
  entry: {
    entry: './entry',
  },
  stats: {
    assets: true,
    modules: true,
  },
});
