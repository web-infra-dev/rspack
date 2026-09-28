import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: {
      import: './a.js',
      chunkLoading: 'async-node',
    },
    b: {
      import: './b.js',
      chunkLoading: 'require',
    },
  },
  output: {
    filename: '[name].js',
  },
});
