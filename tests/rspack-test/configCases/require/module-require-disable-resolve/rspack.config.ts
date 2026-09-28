import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  module: {
    parser: {
      javascript: {
        createRequire: true,
        requireResolve: false,
      },
    },
  },
  optimization: {
    moduleIds: 'named',
  },
});
