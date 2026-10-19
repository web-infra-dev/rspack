import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  experiments: {
    deferImport: true,
  },
  optimization: {
    usedExports: false,
    sideEffects: 'flag',
    concatenateModules: false,
  },
});
