import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: {
    index: './index.js',
    sut: './sut.js',
  },
  output: {
    pathinfo: true,
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    concatenateModules: false,
  },
});
