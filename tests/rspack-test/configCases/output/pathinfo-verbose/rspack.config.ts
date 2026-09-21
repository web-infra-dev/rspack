import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: {
    index: './index.js',
    sut: './sut.js',
  },
  output: {
    pathinfo: 'verbose',
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
    chunkIds: 'named',
    concatenateModules: false,
  },
});
