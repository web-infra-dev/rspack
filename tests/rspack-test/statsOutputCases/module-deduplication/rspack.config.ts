import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    e1: './e1',
    e2: './e2',
    e3: './e3',
  },
  output: {
    filename: '[name].js',
  },
  stats: {
    assets: true,
    hash: false,
    timings: false,
    builtAt: false,
    chunks: true,
    chunkModules: true,
    dependentModules: true,
    modules: false,
  },
});
