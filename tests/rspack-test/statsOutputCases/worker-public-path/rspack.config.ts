import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index.js',
  output: {
    compareBeforeEmit: false,
    filename: '[name]-[contenthash].js',
  },
  stats: {
    assets: true,
    modules: true,
  },
});
