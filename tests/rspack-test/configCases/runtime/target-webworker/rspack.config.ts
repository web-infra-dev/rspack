import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'webworker',
  devtool: false,
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
});
