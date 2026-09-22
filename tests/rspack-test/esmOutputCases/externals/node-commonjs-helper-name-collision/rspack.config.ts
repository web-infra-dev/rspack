import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  module: {
    parser: {
      javascript: {
        createRequire: false,
      },
    },
  },
});
