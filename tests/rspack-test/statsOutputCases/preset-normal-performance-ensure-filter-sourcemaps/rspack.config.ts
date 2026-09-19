import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  devtool: 'source-map',
  performance: {
    hints: 'warning',
    maxAssetSize: 200 * 1024,
    maxEntrypointSize: 200 * 1024,
  },
  entry: './index',
  stats: {
    assets: true,
    modules: true,
    hash: false,
    colors: true,
  },
});
