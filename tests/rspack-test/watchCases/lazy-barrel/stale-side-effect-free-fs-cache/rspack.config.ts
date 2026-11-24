import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: {
    type: 'persistent',
  },
  optimization: {
    sideEffects: true,
  },
});
