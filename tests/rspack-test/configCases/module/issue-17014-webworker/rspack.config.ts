import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
  },
  target: ['web', 'es2020'],
  optimization: {
    splitChunks: {
      minSize: 1,
      maxSize: 1,
    },
  },
});
