import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
  },
  mode: 'production',
  entry: './index',
  stats: {
    assets: true,
    modules: true,
  },
});
