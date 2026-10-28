import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    './the-chunk.js': 'commonjs ./the-chunk.js',
  },
  target: 'web',
  output: {
    chunkFilename: '[name].js',
    uniqueName: 'my "app"',
  },
  performance: {
    hints: false,
  },
  optimization: {
    chunkIds: 'named',
    minimize: false,
  },
});
