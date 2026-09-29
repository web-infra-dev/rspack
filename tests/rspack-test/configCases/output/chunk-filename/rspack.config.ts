import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  target: 'node',
  output: {
    filename: '[name].js',
    chunkFilename: 'chunks/async-[name].[chunkhash:8].js',
  },
});
