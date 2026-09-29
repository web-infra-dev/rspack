import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  resolve: {
    importsFields: ['hash-start', '...'],
  },
});
