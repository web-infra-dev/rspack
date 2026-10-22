import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './src/index',
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
    },
  },
});
