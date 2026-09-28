import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    main: './index',
    sec: './index2',
  },
  stats: {
    assets: true,
    modules: true,
    colors: true,
    hash: false,
    entrypoints: true,
  },
  performance: {
    hints: 'error',
    maxAssetSize: 200 * 1024,
    maxEntrypointSize: 200 * 1024,
  },
});
