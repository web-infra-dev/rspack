import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    './chunk1.js': 'commonjs ./chunk1.js',
    './chunk2.js': 'commonjs ./chunk2.js',
  },
  target: 'web',
  output: {
    chunkFilename: '[name].js',
    crossOriginLoading: 'anonymous',
  },
  performance: {
    hints: false,
  },
  optimization: {
    minimize: false,
  },
});
