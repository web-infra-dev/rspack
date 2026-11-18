import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  output: {
    chunkFilename: '[name].js',
  },
  module: {
    parser: { javascript: { strictThisContextOnImports: true } },
  },
  optimization: {
    minimize: true,
    providedExports: true,
    usedExports: true,
    concatenateModules: false,
  },
});
