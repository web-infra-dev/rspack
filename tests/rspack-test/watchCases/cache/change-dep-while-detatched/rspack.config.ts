import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  cache: true,
  optimization: {
    sideEffects: false,
  },
});
