import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  stats: {
    hash: false,
    timings: false,
    builtAt: false,
    assets: false,
    chunks: true,
    chunkRelations: true,
    chunkModules: true,
    dependentModules: true,
    modules: false,
  },
});
