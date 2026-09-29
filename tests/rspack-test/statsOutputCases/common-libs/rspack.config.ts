import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    react: './react',
  },
  optimization: {
    minimize: true,
    chunkIds: 'named',
  },
  stats: {
    assets: true,
    modules: true,
  },
});
