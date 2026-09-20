import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  optimization: {
    splitChunks: {
      minSize: 0,
    },
  },
});
