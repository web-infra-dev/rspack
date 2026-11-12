import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    chunkFilename: '[name].js',
  },
  performance: {
    hints: false,
  },
  optimization: {
    chunkIds: 'named',
    minimize: false,
  },
});
