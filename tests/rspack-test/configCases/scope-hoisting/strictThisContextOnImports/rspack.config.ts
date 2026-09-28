import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      javascript: {
        strictThisContextOnImports: true,
      },
    },
  },
  optimization: {
    concatenateModules: true,
  },
});
