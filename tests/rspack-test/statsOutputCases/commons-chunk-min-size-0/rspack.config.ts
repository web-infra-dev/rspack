import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    'entry-1': './entry-1',
  },
  optimization: {
    splitChunks: {
      minSize: 0,
      chunks: 'all',
      cacheGroups: {
        'vendor-1': {
          test: /modules[\\/][abc]/,
        },
      },
    },
  },
  stats: {
    entrypoints: 'auto',
    assets: true,
    modules: true,
  },
});
