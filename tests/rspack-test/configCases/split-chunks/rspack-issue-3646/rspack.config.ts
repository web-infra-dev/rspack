import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'node',
  entry: {
    main: './src/index.js',
    another: './src/another.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
    },
  },
});
