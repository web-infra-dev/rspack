import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node22',
  mode: 'development',
  cache: false,
  optimization: {
    sideEffects: true,
    concatenateModules: false,
    minimize: false,
  },
});
