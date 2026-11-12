import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      javascript: {
        exprContextCritical: true,
        wrappedContextCritical: true,
      },
    },
  },
});
