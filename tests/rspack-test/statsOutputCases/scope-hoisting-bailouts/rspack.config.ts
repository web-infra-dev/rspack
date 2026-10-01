import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: {
    index: './index.js',
    entry: './entry.js',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  externals: ['external'],
  stats: {
    assets: false,
    modules: true,
    orphanModules: true,
    optimizationBailout: true,
  },
});
