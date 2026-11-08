import { defineConfig } from '@rspack/cli';

export default defineConfig({
  experiments: {
    runtimeMode: 'rspack',
  },
  output: {
    filename: 'main.js',
    chunkFilename: '[name].js',
  },
  optimization: {
    concatenateModules: false,
  },
});
