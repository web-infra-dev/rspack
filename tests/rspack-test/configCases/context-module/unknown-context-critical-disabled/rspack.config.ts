import { defineConfig } from '@rspack/cli';

export default defineConfig({
  amd: false,
  module: {
    parser: {
      javascript: {
        unknownContextCritical: false,
      },
    },
  },
});
