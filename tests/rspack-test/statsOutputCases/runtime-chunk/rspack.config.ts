import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: {
    e1: './e1',
    e2: './e2',
  },
  output: {
    filename: '[name].js',
  },
  stats: {
    hash: false,
    timings: false,
    builtAt: false,
    entrypoints: true,
    assets: false,
    modules: false,
    reasons: false,
  },
  optimization: {
    runtimeChunk: 'multiple',
  },
});
