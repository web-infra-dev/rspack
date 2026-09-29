import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  stats: {
    assets: true,
    modules: true,
  },
});
