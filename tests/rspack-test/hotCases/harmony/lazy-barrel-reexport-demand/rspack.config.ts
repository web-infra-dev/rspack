import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  optimization: {
    usedExports: false,
    sideEffects: 'flag',
    concatenateModules: false,
  },
});
