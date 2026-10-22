import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: 'static/js/bundle.js',
    chunkFilename: 'chunks/[name].js',
    chunkLoading: 'async-node',
  },
});
