import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  optimization: {
    sideEffects: true,
    providedExports: false,
    concatenateModules: false,
  },
});
