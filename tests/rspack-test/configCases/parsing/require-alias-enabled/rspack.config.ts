import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  module: {
    parser: {
      javascript: {
        requireAlias: true,
      },
    },
  },
});
