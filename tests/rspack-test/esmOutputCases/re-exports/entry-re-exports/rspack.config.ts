import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: false,
  module: {
    parser: {
      javascript: {
        importMeta: false,
      },
    },
  },
});
