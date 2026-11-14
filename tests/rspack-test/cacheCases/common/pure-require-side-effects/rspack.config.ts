import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    concatenateModules: false,
    minimize: true,
    sideEffects: true,
  },
  cache: {
    type: 'persistent',
  },
});
