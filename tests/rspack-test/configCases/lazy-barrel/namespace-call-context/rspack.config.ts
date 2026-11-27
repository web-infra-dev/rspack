import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  module: {
    parser: {
      javascript: {
        strictThisContextOnImports: true,
      },
    },
  },
  optimization: {
    sideEffects: true,
    usedExports: true,
    concatenateModules: false,
  },
});
