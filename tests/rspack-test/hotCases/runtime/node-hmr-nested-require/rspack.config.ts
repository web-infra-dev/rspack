import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  output: {
    filename: 'static/js/bundle.js',
    chunkFilename: 'chunks/[name].js',
  },
});
