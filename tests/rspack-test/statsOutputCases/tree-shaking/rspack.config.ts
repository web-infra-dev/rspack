import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  optimization: {
    concatenateModules: false,
  },
  stats: {
    assets: true,
    chunkModules: false,
    modules: true,
    providedExports: true,
    usedExports: true,
  },
});
