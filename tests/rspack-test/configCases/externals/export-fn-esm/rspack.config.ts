import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    environment: {
      logicalAssignment: false,
    },
  },
  externals: {
    module: 'commonjs module',
    fs: 'commonjs fs',
  },
});
